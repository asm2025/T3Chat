use crate::{
    AppState,
    ai::types::{ChatMessage, ChatRequest as AIChatRequest},
    db::prelude::*,
    db::repositories::{IChatRepository, IMessageRepository, IUserApiKeyRepository},
    middleware::auth::AuthenticatedUser,
};
use axum::{
    body::Body,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub chat_id: uuid::Uuid,
    pub message: String,
    pub model_provider: String,
    pub model_id: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stream: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ChatCompletionResponse {
    pub content: String,
    pub model: String,
    pub tokens_used: Option<u32>,
    pub finish_reason: Option<String>,
}

/// Handle non-streaming chat completion
pub async fn chat(
    user: AuthenticatedUser,
    state: State<AppState>,
    Json(payload): Json<ChatRequest>,
) -> Result<Json<ChatCompletionResponse>, StatusCode> {
    // Verify chat belongs to user
    let _chat = state
        .chat_repository
        .get_by_id(payload.chat_id, &user.0.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Get user's API key for the provider
    let provider = match payload.model_provider.as_str() {
        "openai" => AiProvider::OpenAI,
        "anthropic" => AiProvider::Anthropic,
        "google" => AiProvider::Google,
        "deepseek" => AiProvider::DeepSeek,
        "ollama" => AiProvider::Ollama,
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    let api_key = state
        .user_api_key_repository
        .get_default_for_provider(&user.0.id, &provider)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::BAD_REQUEST)?;

    // TODO: Decrypt API key
    // For now, assume encrypted_key is plaintext (not secure!)
    let decrypted_key = api_key.encrypted_key;

    // Get messages for context
    let messages = state
        .message_repository
        .list_by_chat(payload.chat_id, &user.0.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Convert to AI provider format
    let ai_messages: Vec<ChatMessage> = messages
        .into_iter()
        .map(|m| ChatMessage {
            role: m.role,
            content: m.content,
        })
        .collect();

    // Add user's new message
    let mut ai_messages = ai_messages;
    ai_messages.push(ChatMessage {
        role: MessageRole::User,
        content: payload.message.clone(),
    });

    // Create provider instance
    let mut api_keys = std::collections::HashMap::new();
    api_keys.insert(provider.clone(), decrypted_key);
    let provider_manager = crate::ai::manager::ProviderManager::new(api_keys)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let ai_provider = provider_manager
        .get_provider(&provider)
        .ok_or(StatusCode::BAD_REQUEST)?;

    // Call AI provider
    let ai_request = AIChatRequest {
        model: payload.model_id.clone(),
        messages: ai_messages,
        temperature: payload.temperature,
        max_tokens: payload.max_tokens,
        stream: false,
    };

    let ai_response = ai_provider.chat(ai_request).await.map_err(|e| {
        tracing::error!("AI provider error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Save user message
    let user_seq = state
        .message_repository
        .get_next_sequence_number(payload.chat_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    state
        .message_repository
        .create(CreateMessageDto {
            chat_id: payload.chat_id,
            role: MessageRole::User,
            content: payload.message,
            metadata: None,
            parent_message_id: None,
            sequence_number: user_seq,
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Save assistant response
    let assistant_seq = state
        .message_repository
        .get_next_sequence_number(payload.chat_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let assistant_message = state
        .message_repository
        .create(CreateMessageDto {
            chat_id: payload.chat_id,
            role: MessageRole::Assistant,
            content: ai_response.content.clone(),
            metadata: None,
            parent_message_id: None,
            sequence_number: assistant_seq,
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Update token usage if available
    if let Some(tokens) = ai_response.tokens_used {
        state
            .message_repository
            .update_tokens_used(assistant_message.id, tokens as i32, &ai_response.model)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(Json(ChatCompletionResponse {
        content: ai_response.content,
        model: ai_response.model,
        tokens_used: ai_response.tokens_used,
        finish_reason: ai_response.finish_reason,
    }))
}

/// Handle streaming chat completion
pub async fn stream_chat(
    user: AuthenticatedUser,
    state: State<AppState>,
    Json(payload): Json<ChatRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // For now, return a simple non-streaming response
    // Full SSE streaming implementation would go here
    let response = chat(user, state, Json(payload)).await?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&response.0).unwrap()))
        .unwrap())
}
