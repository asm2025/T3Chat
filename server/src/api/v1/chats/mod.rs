pub mod messages;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{AppState, db::prelude::*, db::repositories::{IChatRepository, IMessageRepository}, middleware::auth::AuthenticatedUser};

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub id: Uuid,
    pub user_id: String,
    pub title: String,
    pub model_provider: String,
    pub model_id: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<ChatModel> for ChatResponse {
    fn from(chat: ChatModel) -> Self {
        Self {
            id: chat.id,
            user_id: chat.user_id,
            title: chat.title,
            model_provider: match chat.model_provider {
                AiProvider::OpenAI => "openai".to_string(),
                AiProvider::Anthropic => "anthropic".to_string(),
                AiProvider::Google => "google".to_string(),
                AiProvider::DeepSeek => "deepseek".to_string(),
                AiProvider::Ollama => "ollama".to_string(),
            },
            model_id: chat.model_id,
            created_at: chat.created_at.to_rfc3339(),
            updated_at: chat.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub id: Uuid,
    pub chat_id: Uuid,
    pub role: String,
    pub content: String,
    pub metadata: Option<serde_json::Value>,
    pub parent_message_id: Option<Uuid>,
    pub sequence_number: i32,
    pub created_at: String,
    pub tokens_used: Option<i32>,
    pub model_used: Option<String>,
}

impl From<MessageModel> for MessageResponse {
    fn from(message: MessageModel) -> Self {
        Self {
            id: message.id,
            chat_id: message.chat_id,
            role: match message.role {
                MessageRole::User => "user".to_string(),
                MessageRole::Assistant => "assistant".to_string(),
                MessageRole::System => "system".to_string(),
            },
            content: message.content,
            metadata: message.metadata,
            parent_message_id: message.parent_message_id,
            sequence_number: message.sequence_number,
            created_at: message.created_at.to_rfc3339(),
            tokens_used: message.tokens_used,
            model_used: message.model_used,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ChatWithMessagesResponse {
    #[serde(flatten)]
    pub chat: ChatResponse,
    pub messages: Vec<MessageResponse>,
}

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct ChatListResponse {
    pub data: Vec<ChatResponse>,
    pub total: u64,
}

#[derive(Debug, Deserialize)]
pub struct CreateChatRequest {
    pub title: Option<String>,
    pub model_provider: String,
    pub model_id: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateChatRequest {
    pub title: Option<String>,
}

/// List all chats for the authenticated user
pub async fn list_chats(
    user: AuthenticatedUser,
    state: State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ChatListResponse>, StatusCode> {
    let pagination = params
        .page
        .zip(params.page_size)
        .map(|(page, page_size)| emixdb::prelude::Pagination {
            page,
            page_size,
        });

    let result = state
        .chat_repository
        .list_by_user(&user.0.id, pagination)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ChatListResponse {
        data: result.data.into_iter().map(ChatResponse::from).collect(),
        total: result.total,
    }))
}

/// Create a new chat
pub async fn create_chat(
    user: AuthenticatedUser,
    state: State<AppState>,
    Json(payload): Json<CreateChatRequest>,
) -> Result<Json<ChatResponse>, StatusCode> {
    let provider = match payload.model_provider.as_str() {
        "openai" => AiProvider::OpenAI,
        "anthropic" => AiProvider::Anthropic,
        "google" => AiProvider::Google,
        "deepseek" => AiProvider::DeepSeek,
        "ollama" => AiProvider::Ollama,
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    let title = payload.title.unwrap_or_else(|| "New Chat".to_string());

    let chat = state
        .chat_repository
        .create(CreateChatDto {
            user_id: user.0.id,
            title,
            model_provider: provider,
            model_id: payload.model_id,
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ChatResponse::from(chat)))
}

/// Get a specific chat with its messages
pub async fn get_chat(
    user: AuthenticatedUser,
    state: State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ChatWithMessagesResponse>, StatusCode> {
    let chat = state
        .chat_repository
        .get_by_id(id, &user.0.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let messages = state
        .message_repository
        .list_by_chat(id, &user.0.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ChatWithMessagesResponse {
        chat: ChatResponse::from(chat),
        messages: messages.into_iter().map(MessageResponse::from).collect(),
    }))
}

/// Update a chat
pub async fn update_chat(
    user: AuthenticatedUser,
    state: State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateChatRequest>,
) -> Result<Json<ChatResponse>, StatusCode> {
    let chat = state
        .chat_repository
        .update(id, &user.0.id, UpdateChatDto {
            title: payload.title,
            model_provider: None,
            model_id: None,
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ChatResponse::from(chat)))
}

/// Delete a chat
pub async fn delete_chat(
    user: AuthenticatedUser,
    state: State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    state
        .chat_repository
        .delete(id, &user.0.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

