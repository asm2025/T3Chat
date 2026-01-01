use crate::{
    AppState,
    ai::{
        manager::ProviderWrapper,
        providers::chatllm::ChatLLMProvider,
        types::{ChatMessage, ChatRequest as AIChatRequest, FeatureFlags, ModelParameters},
    },
    db::models::{MessageRole, AiProvider},
    db::prelude::*,
    db::repositories::{
        chat_repository::TChatRepository, user_api_key_repository::TUserApiKeyRepository,
    },
    middleware::auth::AuthenticatedUser,
    utils::encryption,
};
use axum::{
    extract::State,
    http::StatusCode,
    response::{
        IntoResponse, Json, Response,
        sse::{Event, KeepAlive, Sse},
    },
};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{convert::Infallible, sync::Arc};
use utoipa::ToSchema;

/// End-of-stream marker: sequence of non-printable control characters
/// that are extremely unlikely to appear in normal AI responses.
/// Uses: NULL, SOH, STX, ETX, EOT, ENQ, ACK, BEL
const STREAM_END_MARKER: &str = "\u{0000}\u{0001}\u{0002}\u{0003}\u{0004}\u{0005}\u{0006}\u{0007}";

/// Chat completion request with full LibreChat support
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChatRequest {
    /// Chat ID
    #[serde(alias = "chat_id", alias = "id")]
    pub chat_id: uuid::Uuid,
    /// User's message text
    pub message: String,
    /// AI provider (openai, anthropic, google, etc.)
    #[serde(alias = "model_provider")]
    pub model_provider: String,
    /// Model ID (e.g., gpt-4-turbo, claude-3-opus)
    #[serde(alias = "model_id")]
    pub model_id: String,
    /// Model parameters (temperature, max_tokens, etc.)
    #[serde(default)]
    #[serde(alias = "model_parameters")]
    pub model_parameters: Option<Value>,
    /// Feature flags (resend_files, prompt_cache, etc.)
    #[serde(default)]
    #[serde(alias = "feature_flags")]
    pub feature_flags: Option<Value>,
    /// System message/instructions
    #[serde(alias = "system_message")]
    pub system_message: Option<String>,
    /// Enable streaming response
    #[serde(default)]
    pub stream: bool,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChatCompletionResponse {
    pub content: String,
    pub model: String,
    pub usage: Option<TokenUsageResponse>,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsageResponse {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Handle non-streaming chat completion
#[utoipa::path(
    post,
    path = "/api/v1/chat",
    tag = "Chat",
    security(("bearer_auth" = [])),
    request_body = ChatRequest,
    responses(
        (status = 200, description = "Chat completion response", body = ChatCompletionResponse),
        (status = 400, description = "Invalid provider or API key"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Chat not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn chat(
    user: AuthenticatedUser,
    state: State<AppState>,
    Json(payload): Json<ChatRequest>,
) -> Result<Json<ChatCompletionResponse>, Response> {
    // Verify chat belongs to user
    let chat = state
        .chat_repository
        .get(payload.chat_id, &user.0.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?
        .ok_or(StatusCode::NOT_FOUND.into_response())?;

    let provider_key = payload.model_provider.to_lowercase();

    // Verify the provider is configured (has an endpoint in config)
    let endpoint_config = state
        .app_config
        .endpoints
        .iter()
        .find(|e| e.provider.to_lowercase() == provider_key);

    // Log if model isn't in catalog (warning only, don't block - let upstream API handle invalid models)
    if state
        .model_catalog
        .find(&provider_key, &payload.model_id)
        .is_none()
    {
        tracing::warn!(
            "Model not found in catalog (proceeding anyway): provider={}, model={}",
            provider_key,
            payload.model_id
        );
    }

    tracing::info!("Resolving provider: {}", provider_key);

    // Custom providers configured via YAML (e.g., chatllm, openrouter) use backend-managed API keys
    let is_custom_provider = matches!(
        provider_key.as_str(),
        "chatllm" | "openrouter"
    );

    let ai_provider: Arc<ProviderWrapper> = if is_custom_provider {
        // Custom provider - requires endpoint config with API key
        let config = endpoint_config.ok_or_else(|| {
            tracing::error!("Custom provider {} not configured in YAML", provider_key);
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("Provider {} not configured on server", provider_key) })),
            )
                .into_response()
        })?;

        let api_key = config.api_key.clone().ok_or_else(|| {
            tracing::error!("API key missing in config for provider: {}", provider_key);
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("{} API key missing on server", provider_key) })),
            )
                .into_response()
        })?;

        let base_url = config.base_url.clone();

        Arc::new(ProviderWrapper::ChatLLM(ChatLLMProvider::new(
            api_key, base_url,
        )))
    } else {
        // Built-in provider - use user's API key
        let provider = match provider_key.as_str() {
            "openai" => AiProvider::OpenAI,
            "anthropic" => AiProvider::Anthropic,
            "google" => AiProvider::Google,
            "deepseek" => AiProvider::DeepSeek,
            "ollama" => AiProvider::Ollama,
            _ => {
                tracing::error!("Unknown provider: {}", provider_key);
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": format!("Unknown provider: {}", provider_key) }))
                ).into_response());
            }
        };

        let api_key = state
            .user_api_key_repository
            .get_default_for_provider(&user.0.id, &provider)
            .await
            .map_err(|e| {
                tracing::error!("Database error fetching API key: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            })?
            .ok_or_else(|| {
                tracing::warn!("No API key found for provider: {:?}", provider);
                (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ 
                        "error": format!("Missing API key for provider '{}'. Please add one in Settings > API Keys.", provider.as_str()),
                        "code": "missing_api_key",
                        "provider": provider.as_str()
                    }))
                ).into_response()
            })?;

        // Decrypt API key
        let decrypted_key = encryption::decrypt(&api_key.encrypted_key).map_err(|e| {
            tracing::error!("Failed to decrypt API key: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        })?;

        let mut api_keys = std::collections::HashMap::new();
        api_keys.insert(provider.clone(), decrypted_key);

        // Add ABACUS_API_KEY from env if available for ChatLLM
        if let Some(abacus_key) = crate::env::get_abacus_api_key() {
            api_keys.insert(AiProvider::ChatLLM, abacus_key);
        }

        let provider_manager = crate::ai::manager::ProviderManager::new(api_keys)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

        provider_manager
            .get_provider(&provider)
            .ok_or((StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Failed to initialize provider" }))).into_response())?
    };

    // Get messages for context
    let messages = state
        .chat_repository
        .list_messages(chat.id, &user.0.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

    // Convert to AI provider format
    let ai_messages: Vec<ChatMessage> = messages
        .into_iter()
        .filter_map(|m| {
            MessageRole::from_str(&m.role).map(|role| ChatMessage {
                role,
                content: m.text.unwrap_or_default(),
                name: None,
            })
        })
        .collect();

    // Add user's new message
    let mut ai_messages = ai_messages;
    ai_messages.push(ChatMessage {
        role: MessageRole::User,
        content: payload.message.clone(),
        name: None,
    });

    // Parse model parameters from request or use defaults
    let parameters: ModelParameters = payload
        .model_parameters
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    // Parse feature flags from request
    let feature_flags: Option<FeatureFlags> = payload
        .feature_flags
        .and_then(|v| serde_json::from_value(v).ok());

    // Call AI provider
    let ai_request = AIChatRequest {
        model: payload.model_id.clone(),
        messages: ai_messages,
        parameters,
        feature_flags,
        system_message: payload.system_message.clone(),
        stream: false,
    };

    let ai_response = ai_provider.chat(ai_request).await.map_err(|e| {
        tracing::error!("AI provider error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    })?;

    // Save user message
    let user_seq = state
        .chat_repository
        .get_next_sequence_number(chat.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

    state
        .chat_repository
        .create_message(CreateMessageDto {
            chat_id: chat.id,
            role: MessageRole::User,
            content: payload.message,
            metadata: None,
            parent_message_id: None,
            sequence_number: user_seq,
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

    // Save assistant response
    let assistant_seq = state
        .chat_repository
        .get_next_sequence_number(chat.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

    let assistant_message = state
        .chat_repository
        .create_message(CreateMessageDto {
            chat_id: chat.id,
            role: MessageRole::Assistant,
            content: ai_response.content.clone(),
            metadata: None,
            parent_message_id: None,
            sequence_number: assistant_seq,
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

    // Update token usage if available
    if let Some(usage) = &ai_response.usage {
        state
            .chat_repository
            .update_tokens_used(
                assistant_message.id,
                usage.total_tokens as i32,
                &ai_response.model,
            )
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;
    }

    Ok(Json(ChatCompletionResponse {
        content: ai_response.content,
        model: ai_response.model,
        usage: ai_response.usage.map(|u| TokenUsageResponse {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        }),
        finish_reason: ai_response.finish_reason,
    }))
}

/// Handle streaming chat completion via Server-Sent Events
#[utoipa::path(
    post,
    path = "/api/v1/chat/stream",
    tag = "Chat",
    security(("bearer_auth" = [])),
    request_body = ChatRequest,
    responses(
        (status = 200, description = "Streaming chat completion response"),
        (status = 400, description = "Invalid provider or API key"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Chat not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn stream_chat(
    user: AuthenticatedUser,
    state: State<AppState>,
    Json(payload): Json<ChatRequest>,
) -> Result<Sse<impl futures::Stream<Item = Result<Event, Infallible>>>, Response> {
    tracing::info!("Stream chat request: provider={}, model={}", payload.model_provider, payload.model_id);

    // Verify chat belongs to user
    let chat = state
        .chat_repository
        .get(payload.chat_id, &user.0.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?
        .ok_or(StatusCode::NOT_FOUND.into_response())?;

    let provider_key = payload.model_provider.to_lowercase();

    // Verify the provider is configured (has an endpoint in config)
    let endpoint_config = state
        .app_config
        .endpoints
        .iter()
        .find(|e| e.provider.to_lowercase() == provider_key);

    // Log if model isn't in catalog (warning only, don't block - let upstream API handle invalid models)
    if state
        .model_catalog
        .find(&provider_key, &payload.model_id)
        .is_none()
    {
        tracing::warn!(
            "Model not found in catalog (proceeding anyway): provider={}, model={}",
            provider_key,
            payload.model_id
        );
    }

    tracing::info!("Resolving provider: {}", provider_key);

    // Custom providers configured via YAML (e.g., chatllm, openrouter) use backend-managed API keys
    let is_custom_provider = matches!(
        provider_key.as_str(),
        "chatllm" | "openrouter"
    );

    let ai_provider: Arc<ProviderWrapper> = if is_custom_provider {
        // Custom provider - requires endpoint config with API key
        let config = endpoint_config.ok_or_else(|| {
            tracing::error!("Custom provider {} not configured in YAML", provider_key);
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("Provider {} not configured on server", provider_key) })),
            )
                .into_response()
        })?;

        let api_key = config.api_key.clone().ok_or_else(|| {
            tracing::error!("API key missing in config for provider: {}", provider_key);
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("{} API key missing on server", provider_key) })),
            )
                .into_response()
        })?;

        let base_url = config.base_url.clone();

        Arc::new(ProviderWrapper::ChatLLM(ChatLLMProvider::new(
            api_key, base_url,
        )))
    } else {
        // Built-in provider - use user's API key
        let provider = match provider_key.as_str() {
            "openai" => AiProvider::OpenAI,
            "anthropic" => AiProvider::Anthropic,
            "google" => AiProvider::Google,
            "deepseek" => AiProvider::DeepSeek,
            "ollama" => AiProvider::Ollama,
            _ => {
                tracing::error!("Unknown provider: {}", provider_key);
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": format!("Unknown provider: {}", provider_key) }))
                ).into_response());
            }
        };

        let api_key = state
            .user_api_key_repository
            .get_default_for_provider(&user.0.id, &provider)
            .await
            .map_err(|e| {
                tracing::error!("Database error fetching API key: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            })?
            .ok_or_else(|| {
                tracing::warn!("No API key found for provider: {:?}", provider);
                (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ 
                        "error": format!("Missing API key for provider '{}'. Please add one in Settings > API Keys.", provider.as_str()),
                        "code": "missing_api_key",
                        "provider": provider.as_str()
                    }))
                ).into_response()
            })?;

        // Decrypt API key
        let decrypted_key = encryption::decrypt(&api_key.encrypted_key).map_err(|e| {
            tracing::error!("Failed to decrypt API key: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        })?;

        let mut api_keys = std::collections::HashMap::new();
        api_keys.insert(provider.clone(), decrypted_key);

        // Add ABACUS_API_KEY from env if available for ChatLLM
        if let Some(abacus_key) = crate::env::get_abacus_api_key() {
            api_keys.insert(AiProvider::ChatLLM, abacus_key);
        }

        let provider_manager = crate::ai::manager::ProviderManager::new(api_keys)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

        provider_manager
            .get_provider(&provider)
            .ok_or((StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Failed to initialize provider" }))).into_response())?
    };

    // Get messages for context
    let messages = state
        .chat_repository
        .list_messages(chat.id, &user.0.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

    // Convert to AI provider format
    let ai_messages: Vec<ChatMessage> = messages
        .into_iter()
        .filter_map(|m| {
            MessageRole::from_str(&m.role).map(|role| ChatMessage {
                role,
                content: m.text.unwrap_or_default(),
                name: None,
            })
        })
        .collect();

    // Add user's new message
    let mut ai_messages = ai_messages;
    ai_messages.push(ChatMessage {
        role: MessageRole::User,
        content: payload.message.clone(),
        name: None,
    });

    // Parse model parameters
    let parameters: ModelParameters = payload
        .model_parameters
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    let feature_flags: Option<FeatureFlags> = payload
        .feature_flags
        .and_then(|v| serde_json::from_value(v).ok());

    // Create streaming request
    let ai_request = AIChatRequest {
        model: payload.model_id.clone(),
        messages: ai_messages,
        parameters,
        feature_flags,
        system_message: payload.system_message,
        stream: true,
    };

    // Get streaming response
    let mut stream = ai_provider.stream_chat(ai_request).await.map_err(|e| {
        tracing::error!("AI provider streaming error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    })?;

    // Save user message
    let chat_repo = state.chat_repository.clone();
    let chat_id = chat.id;
    let user_message_content = payload.message.clone();

    let user_seq = state
        .chat_repository
        .get_next_sequence_number(chat_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

    let user_msg = state
        .chat_repository
        .create_message(CreateMessageDto {
            chat_id,
            role: MessageRole::User,
            content: user_message_content,
            metadata: None,
            parent_message_id: None,
            sequence_number: user_seq,
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

    // Create SSE stream
    let mut full_response = String::new();
    let mut _finish_reason: Option<String> = None;
    let model = payload.model_id.clone();
    let user_msg_id = user_msg.id;

    let event_stream = async_stream::stream! {
        let mut stream_complete = false;
        
        // Process all chunks and accumulate the full response
        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    full_response.push_str(&chunk.delta);
                    // Send chunk as SSE event
                    let event = Event::default()
                        .json_data(&chunk)
                        .unwrap_or_else(|_| Event::default().data("error"));

                    yield Ok::<Event, Infallible>(event);

                    // Mark stream as complete when done, but don't save yet
                    if chunk.done {
                        stream_complete = true;
                    }
                }
                Err(e) => {
                    tracing::error!("Stream error: {}", e);

                    // Match LibreChat's structured error format
                    let error_payload = serde_json::json!({
                        "error": true,
                        "messageId": uuid::Uuid::new_v4().to_string(),
                        "chatId": chat_id,
                        "parentMessageId": user_msg_id,
                        "sender": "Assistant",
                        "text": format!("Error: {}", e),
                        "final": true,
                        "unfinished": false
                    });

                    let error_event = Event::default()
                        .event("error")
                        .data(error_payload.to_string());
                    yield Ok(error_event);
                    break;
                }
            }
        }
        
        // Save assistant message ONCE after all chunks are processed
        if stream_complete && !full_response.is_empty() {
            let assistant_seq = chat_repo
                .get_next_sequence_number(chat_id)
                .await
                .unwrap_or(user_seq + 1);

            let assistant_message = chat_repo
                .create_message(CreateMessageDto {
                    chat_id,
                    role: MessageRole::Assistant,
                    content: full_response.clone(),
                    metadata: None,
                    parent_message_id: None,
                    sequence_number: assistant_seq,
                })
                .await;

            if let Ok(msg) = assistant_message {
                // Update with model info if available
                let _ = chat_repo
                    .update_tokens_used(msg.id, 0, &model)
                    .await;
            }
        }
        
        // Send final end-of-stream marker after all chunks are processed
        if stream_complete {
            let done_event = Event::default().data(STREAM_END_MARKER);
            yield Ok::<Event, Infallible>(done_event);
        }
    };

    Ok(Sse::new(event_stream).keep_alive(KeepAlive::default()))
}
