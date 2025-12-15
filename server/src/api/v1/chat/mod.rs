use crate::{
    AppState,
    ai::{
        manager::ProviderWrapper,
        providers::routellm::RouteLLMProvider,
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

/// Chat completion request with full LibreChat support
#[derive(Debug, Deserialize, ToSchema)]
pub struct ChatRequest {
    /// Chat ID
    pub chat_id: uuid::Uuid,
    /// User's message text
    pub message: String,
    /// AI provider (openai, anthropic, google, etc.)
    pub model_provider: String,
    /// Model ID (e.g., gpt-4-turbo, claude-3-opus)
    pub model_id: String,
    /// Model parameters (temperature, max_tokens, etc.)
    #[serde(default)]
    pub model_parameters: Option<Value>,
    /// Feature flags (resend_files, prompt_cache, etc.)
    #[serde(default)]
    pub feature_flags: Option<Value>,
    /// System message/instructions
    pub system_message: Option<String>,
    /// Enable streaming response
    #[serde(default)]
    pub stream: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ChatCompletionResponse {
    pub content: String,
    pub model: String,
    pub usage: Option<TokenUsageResponse>,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
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
    let _chat = state
        .chat_repository
        .get(payload.chat_id, &user.0.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?
        .ok_or(StatusCode::NOT_FOUND.into_response())?;

    if state
        .model_catalog
        .find(&payload.model_provider, &payload.model_id)
        .is_none()
    {
        tracing::error!("Model not found in catalog: provider={}, model={}", payload.model_provider, payload.model_id);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": format!("Model not found: {}/{}", payload.model_provider, payload.model_id) }))
        ).into_response());
    }

    // Resolve provider implementation
    let provider_name = payload.model_provider.as_str();
    tracing::info!("Resolving provider: {}", provider_name);

    // Special handling for RouteLLM: uses a backend-managed key from librechat.yaml (DerivedAppConfig)
    let ai_provider: Arc<ProviderWrapper> = if provider_name == "routellm" {
        // Find routellm config in endpoints
        let routellm_endpoint = state.app_config.endpoints.iter().find(|e| e.provider == "routellm");
        
        let api_key = routellm_endpoint
            .and_then(|e| e.api_key.clone())
            .ok_or_else(|| {
                 tracing::error!("RouteLLM API key missing in config");
                 (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": "RouteLLM API key missing on server" }))
                 ).into_response()
            })?;
            
        let base_url = routellm_endpoint.and_then(|e| e.base_url.clone());

        Arc::new(ProviderWrapper::RouteLLM(RouteLLMProvider::new(
            api_key, base_url,
        )))
    } else {
        // Get user's API key for the provider
        let provider = match provider_name {
            "openai" => AiProvider::OpenAI,
            "anthropic" => AiProvider::Anthropic,
            "google" => AiProvider::Google,
            "deepseek" => AiProvider::DeepSeek,
            "ollama" => AiProvider::Ollama,
            _ => {
                tracing::error!("Invalid provider name: {}", provider_name);
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": format!("Invalid provider: {}", provider_name) }))
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
        .list_messages(payload.chat_id, &user.0.id)
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
        .get_next_sequence_number(payload.chat_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

    state
        .chat_repository
        .create_message(CreateMessageDto {
            chat_id: payload.chat_id,
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
        .get_next_sequence_number(payload.chat_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

    let assistant_message = state
        .chat_repository
        .create_message(CreateMessageDto {
            chat_id: payload.chat_id,
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
    let _chat = state
        .chat_repository
        .get(payload.chat_id, &user.0.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?
        .ok_or(StatusCode::NOT_FOUND.into_response())?;

    if state
        .model_catalog
        .find(&payload.model_provider, &payload.model_id)
        .is_none()
    {
        tracing::error!("Model not found in catalog: provider={}, model={}", payload.model_provider, payload.model_id);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": format!("Model not found: {}/{}", payload.model_provider, payload.model_id) }))
        ).into_response());
    }

    // Resolve provider implementation
    let provider_name = payload.model_provider.as_str();
    tracing::info!("Resolving provider: {}", provider_name);

    // Special handling for RouteLLM: uses a backend-managed key from librechat.yaml (DerivedAppConfig)
    let ai_provider: Arc<ProviderWrapper> = if provider_name == "routellm" {
        // Find routellm config in endpoints
        let routellm_endpoint = state.app_config.endpoints.iter().find(|e| e.provider == "routellm");
        
        let api_key = routellm_endpoint
            .and_then(|e| e.api_key.clone())
            .ok_or_else(|| {
                 tracing::error!("RouteLLM API key missing in config");
                 (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": "RouteLLM API key missing on server" }))
                 ).into_response()
            })?;
            
        let base_url = routellm_endpoint.and_then(|e| e.base_url.clone());

        Arc::new(ProviderWrapper::RouteLLM(RouteLLMProvider::new(
            api_key, base_url,
        )))
    } else {
        // Get user's API key for the provider
        let provider = match provider_name {
            "openai" => AiProvider::OpenAI,
            "anthropic" => AiProvider::Anthropic,
            "google" => AiProvider::Google,
            "deepseek" => AiProvider::DeepSeek,
            "ollama" => AiProvider::Ollama,
            _ => {
                tracing::error!("Invalid provider name: {}", provider_name);
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": format!("Invalid provider: {}", provider_name) }))
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
        .list_messages(payload.chat_id, &user.0.id)
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
    let chat_id = payload.chat_id;
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
        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    full_response.push_str(&chunk.delta);
                    if chunk.done {
                        _finish_reason = chunk.finish_reason.clone();
                    }

                    // Send chunk as SSE event
                    let event = Event::default()
                        .json_data(&chunk)
                        .unwrap_or_else(|_| Event::default().data("error"));

                    yield Ok::<Event, Infallible>(event);

                    // If done, save assistant response
                    if chunk.done {
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

                        break;
                    }
                }
                Err(e) => {
                    tracing::error!("Stream error: {}", e);

                    // Match LibreChat's structured error format
                    let error_payload = serde_json::json!({
                        "error": true,
                        "messageId": uuid::Uuid::new_v4().to_string(),
                        "conversationId": chat_id,
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
    };

    Ok(Sse::new(event_stream).keep_alive(KeepAlive::default()))
}
