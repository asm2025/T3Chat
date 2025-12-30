pub mod messages;

use crate::{
    AppState, db::dto::Pagination, db::prelude::*,
    db::repositories::chat_repository::TChatRepository, middleware::auth::AuthenticatedUser,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChatResponse {
    pub id: Uuid,
    pub user_id: String,
    pub title: String,
    pub model_provider: String,
    pub model_id: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Chat> for ChatResponse {
    fn from(chat: Chat) -> Self {
        // Parse provider from endpoint string
        let model_provider = AiProvider::from_str(&chat.endpoint).unwrap_or(AiProvider::OpenAI);
        Self {
            id: chat.id,
            user_id: chat.user_id,
            title: chat.title.unwrap_or_else(|| "New Chat".to_string()),
            model_provider: model_provider.as_str().to_string(),
            model_id: chat.model,
            created_at: chat.created_at.to_rfc3339(),
            updated_at: chat.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
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

// Convert from the chat::Message model
impl From<Message> for MessageResponse {
    fn from(message: Message) -> Self {
        Self {
            id: message.id,
            chat_id: message.chat_id, // Note: chat_id maps to chat_id for API compatibility
            role: message.role,
            content: message.text.unwrap_or_default(),
            metadata: message.content, // content field stores metadata JSON
            parent_message_id: message.parent_message_id,
            sequence_number: 0, // Not stored in new model, would need to be calculated
            created_at: message.created_at.to_rfc3339(),
            tokens_used: message.token_count,
            model_used: message.model,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChatWithMessagesResponse {
    pub chat: ChatResponse,
    pub messages: Vec<MessageResponse>,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
#[serde(rename_all = "camelCase")]
pub struct PaginationParams {
    pub page: Option<u64>,
    #[serde(alias = "page_size")]
    pub page_size: Option<u64>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChatListResponse {
    pub data: Vec<ChatResponse>,
    pub total: u64,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateChatRequest {
    pub title: Option<String>,
    #[serde(alias = "model_provider")]
    pub model_provider: String,
    #[serde(alias = "model_id")]
    pub model_id: String,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateChatRequest {
    pub title: Option<String>,
}

/// List all chats for the authenticated user
#[utoipa::path(
    get,
    path = "/api/v1/chats",
    tag = "Chats",
    params(PaginationParams),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of chats", body = ChatListResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn list_chats(
    user: AuthenticatedUser,
    state: State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ChatListResponse>, StatusCode> {
    let pagination = params
        .page
        .zip(params.page_size)
        .map(|(page, page_size)| Pagination { page, page_size });

    let result = state
        .chat_repository
        .list(&user.0.id, pagination)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ChatListResponse {
        data: result.data.into_iter().map(ChatResponse::from).collect(),
        total: result.total,
    }))
}

/// Create a new chat
#[utoipa::path(
    post,
    path = "/api/v1/chats",
    tag = "Chats",
    security(("bearer_auth" = [])),
    request_body = CreateChatRequest,
    responses(
        (status = 200, description = "Chat created", body = ChatResponse),
        (status = 400, description = "Invalid provider or payload"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_chat(
    user: AuthenticatedUser,
    state: State<AppState>,
    Json(payload): Json<CreateChatRequest>,
) -> Result<Json<ChatResponse>, StatusCode> {
    let provider = AiProvider::from_str(&payload.model_provider)
        .ok_or_else(|| {
            tracing::warn!("Unknown provider: {}", payload.model_provider);
            StatusCode::BAD_REQUEST
        })?;

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
        .map_err(|e| {
            tracing::error!("Failed to create chat: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(ChatResponse::from(chat)))
}

/// Get a specific chat with its messages
#[utoipa::path(
    get,
    path = "/api/v1/chats/{id}",
    tag = "Chats",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Chat identifier")
    ),
    responses(
        (status = 200, description = "Chat detail", body = ChatWithMessagesResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Chat not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_chat(
    user: AuthenticatedUser,
    state: State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ChatWithMessagesResponse>, StatusCode> {
    let chat = state
        .chat_repository
        .get(id, &user.0.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let messages = state
        .chat_repository
        .list_messages(chat.id, &user.0.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ChatWithMessagesResponse {
        chat: ChatResponse::from(chat),
        messages: messages.into_iter().map(MessageResponse::from).collect(),
    }))
}

/// Update a chat
#[utoipa::path(
    put,
    path = "/api/v1/chats/{id}",
    tag = "Chats",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Chat identifier")
    ),
    request_body = UpdateChatRequest,
    responses(
        (status = 200, description = "Chat updated", body = ChatResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Chat not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn update_chat(
    user: AuthenticatedUser,
    state: State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateChatRequest>,
) -> Result<Json<ChatResponse>, StatusCode> {
    // Resolve `id` which might be either the internal UUID or the external chat_id (TEXT)
    let chat = state
        .chat_repository
        .get(id, &user.0.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let chat = state
        .chat_repository
        .update(
            chat.id,
            &user.0.id,
            UpdateChatDto {
                title: payload.title,
                model_provider: None,
                model_id: None,
            },
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ChatResponse::from(chat)))
}

/// Delete a chat
#[utoipa::path(
    delete,
    path = "/api/v1/chats/{id}",
    tag = "Chats",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Chat identifier")
    ),
    responses(
        (status = 204, description = "Chat deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Chat not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn delete_chat(
    user: AuthenticatedUser,
    state: State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    // Resolve `id` which might be either the internal UUID or the external chat_id (TEXT)
    let chat = state
        .chat_repository
        .get(id, &user.0.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    state
        .chat_repository
        .delete(chat.id, &user.0.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}
