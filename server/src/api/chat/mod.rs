use crate::{
    AppState,
    ai::{
        manager::ProviderWrapper,
        providers::chatllm::ChatLLMProvider,
        types::{AgentEvent, ChatMessage, ChatRequest as AIChatRequest, FeatureFlags, ModelParameters},
    },
    db::models::{AiProvider, MessageRole, NewToolCall, UpdateMessageDto, UpdateToolCall},
    db::prelude::*,
    db::repositories::{
        chat_repository::TChatRepository, tool_call_repository::TToolCallRepository,
        user_api_key_repository::TUserApiKeyRepository,
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
use chrono::Utc;
use std::{collections::HashMap, convert::Infallible, sync::Arc, time::Instant};
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
    path = "/api/chat",
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
    path = "/api/chat/stream",
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

    // Resolve available tools for the chat's agent/assistant
    let available_tools = state
        .tool_executor
        .get_available_tools(chat.agent_id, chat.assistant_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to resolve tools: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        })?;

    // Get streaming response with tool support
    let mut stream = ai_provider
        .stream_chat_with_tools(ai_request, available_tools)
        .await
        .map_err(|e| {
            tracing::error!("AI provider streaming error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        })?;

    // Save user message
    let chat_id = chat.id;
    let user_message_content = payload.message.clone();

    let user_seq = state
        .chat_repository
        .get_next_sequence_number(chat_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

    state
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
    let model = payload.model_id.clone();
    let user_id = user.0.id.clone();
    let chat_repo = state.chat_repository.clone();
    let tool_call_repo = state.tool_call_repository.clone();

    let event_stream = async_stream::stream! {
        let mut stream_complete = false;
        let mut assistant_message_id: Option<uuid::Uuid> = None;
        let mut full_response = String::new();
        let mut usage: Option<crate::ai::types::TokenUsage> = None;
        let mut tool_start_times: HashMap<String, Instant> = HashMap::new();

        while let Some(event_result) = stream.next().await {
            match event_result {
                Ok(event) => {
                    match &event {
                        AgentEvent::TextDelta { delta } => {
                            full_response.push_str(delta);
                        }
                        AgentEvent::MessageComplete { content, finish_reason: _reason, usage: event_usage } => {
                            full_response = content.clone();
                            usage = event_usage.clone();
                            stream_complete = true;
                        }
                        AgentEvent::ToolStart { tool_call_id, tool_name, tool_type, arguments, .. } => {
                            if assistant_message_id.is_none() {
                                let assistant_seq = chat_repo
                                    .get_next_sequence_number(chat_id)
                                    .await
                                    .unwrap_or(user_seq + 1);

                                if let Ok(msg) = chat_repo
                                    .create_message(CreateMessageDto {
                                        chat_id,
                                        role: MessageRole::Assistant,
                                        content: String::new(),
                                        metadata: None,
                                        parent_message_id: None,
                                        sequence_number: assistant_seq,
                                    })
                                    .await
                                {
                                    assistant_message_id = Some(msg.id);
                                }
                            }

                            if let Some(message_id) = assistant_message_id {
                                tool_start_times.insert(tool_call_id.clone(), Instant::now());
                                let new_tool_call = NewToolCall {
                                    id: None,
                                    message_id,
                                    tool_call_id: tool_call_id.clone(),
                                    tool_name: tool_name.clone(),
                                    tool_type: Some(tool_type.clone()),
                                    arguments: Some(arguments.clone()),
                                    result: None,
                                    status: Some("running".to_string()),
                                    started_at: Some(Utc::now()),
                                    execution_time_ms: None,
                                };

                                if let Err(e) = tool_call_repo.create(new_tool_call).await {
                                    tracing::warn!("Failed to create tool call: {}", e);
                                }
                            }
                        }
                        AgentEvent::ToolEnd { tool_call_id, status, error, result, .. } => {
                            if let Some(message_id) = assistant_message_id {
                                let execution_time_ms = tool_start_times
                                    .remove(tool_call_id)
                                    .and_then(|start| i64::try_from(start.elapsed().as_millis()).ok());
                                let update = UpdateToolCall {
                                    result: result.clone(),
                                    status: Some(status.clone()),
                                    error_message: Some(error.clone()),
                                    output_file_ids: None,
                                    execution_time_ms,
                                    completed_at: Some(Utc::now()),
                                    updated_at: Utc::now(),
                                };

                                if let Err(e) = tool_call_repo
                                    .update_by_tool_call_id(message_id, tool_call_id, update)
                                    .await
                                {
                                    tracing::warn!("Failed to update tool call: {}", e);
                                }
                            }
                        }
                        _ => {}
                    }

                    let json = serde_json::to_string(&event).unwrap_or_else(|_| "{\"type\":\"error\",\"message\":\"serialization_error\"}".to_string());
                    let sse_event = Event::default().data(json);
                    yield Ok::<Event, Infallible>(sse_event);
                }
                Err(e) => {
                    tracing::error!("Stream error: {}", e);
                    let error_event = AgentEvent::Error {
                        message: format!("Error: {}", e),
                        code: None,
                    };
                    let json = serde_json::to_string(&error_event).unwrap_or_else(|_| "{\"type\":\"error\",\"message\":\"stream_error\"}".to_string());
                    yield Ok(Event::default().data(json));
                    break;
                }
            }
        }

        if !stream_complete && !full_response.is_empty() {
            let complete_event = AgentEvent::MessageComplete {
                content: full_response.clone(),
                finish_reason: None,
                usage: usage.clone(),
            };
            let json = serde_json::to_string(&complete_event)
                .unwrap_or_else(|_| "{\"type\":\"message_complete\",\"content\":\"\"}".to_string());
            yield Ok(Event::default().data(json));
            stream_complete = true;
        }

        if stream_complete && !full_response.is_empty() {
            let assistant_message = if let Some(message_id) = assistant_message_id {
                chat_repo
                    .update_message(
                        message_id,
                        chat_id,
                        &user_id,
                        UpdateMessageDto {
                            content: Some(full_response.clone()),
                            metadata: None,
                        },
                    )
                    .await
            } else {
                let assistant_seq = chat_repo
                    .get_next_sequence_number(chat_id)
                    .await
                    .unwrap_or(user_seq + 1);

                chat_repo
                    .create_message(CreateMessageDto {
                        chat_id,
                        role: MessageRole::Assistant,
                        content: full_response.clone(),
                        metadata: None,
                        parent_message_id: None,
                        sequence_number: assistant_seq,
                    })
                    .await
            };

            if let Ok(msg) = assistant_message {
                let tokens = usage.as_ref().map(|u| u.total_tokens as i32).unwrap_or(0);
                let _ = chat_repo
                    .update_tokens_used(msg.id, tokens, &model)
                    .await;

                let meili = state.meilisearch.clone();
                let message_doc = crate::utils::meilisearch::MeiliDocument {
                    id: format!("message:{}", msg.id),
                    user_id: user_id.clone(),
                    chat_id: msg.chat_id.to_string(),
                    r#type: "message".to_string(),
                    title: None,
                    text: msg.text.clone(),
                    role: Some(msg.role.clone()),
                    model: msg.model.clone(),
                    endpoint: None,
                    created_at: msg.created_at.to_rfc3339(),
                    updated_at: msg.updated_at.to_rfc3339(),
                };
                tokio::spawn(async move {
                    if let Err(e) = meili.index_message(message_doc).await {
                        tracing::warn!("Failed to index message in MeiliSearch: {}", e);
                    }
                });
            }
        }

        if stream_complete {
            let done_event = Event::default().data(STREAM_END_MARKER);
            yield Ok::<Event, Infallible>(done_event);
        }
    };

    Ok(Sse::new(event_stream).keep_alive(KeepAlive::default()))
}
