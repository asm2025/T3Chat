use super::MessageResponse;
use crate::{
    AppState,
    db::prelude::*,
    db::repositories::TChatRepository,
    middleware::auth::AuthenticatedUser,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateMessageRequest {
    pub content: String,
    pub role: Option<String>,
}

/// Get all messages for a chat
#[utoipa::path(
    get,
    path = "/api/v1/chats/{id}/messages",
    tag = "Messages",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Chat identifier")
    ),
    responses(
        (status = 200, description = "Messages list", body = [super::MessageResponse]),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Chat not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_messages(
    user: AuthenticatedUser,
    state: State<AppState>,
    Path(chat_id): Path<Uuid>,
) -> Result<Json<Vec<MessageResponse>>, StatusCode> {
    let messages = state
        .chat_repository
        .list_messages(chat_id, &user.0.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(
        messages.into_iter().map(MessageResponse::from).collect(),
    ))
}

/// Create a new message in a chat
#[utoipa::path(
    post,
    path = "/api/v1/chats/{id}/messages",
    tag = "Messages",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Chat identifier")
    ),
    request_body = CreateMessageRequest,
    responses(
        (status = 200, description = "Message created", body = super::MessageResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Chat not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_message(
    user: AuthenticatedUser,
    state: State<AppState>,
    Path(chat_id): Path<Uuid>,
    Json(payload): Json<CreateMessageRequest>,
) -> Result<Json<MessageResponse>, StatusCode> {
    // Verify chat belongs to user
    let _chat = state
        .chat_repository
        .get(chat_id, &user.0.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let role = match payload.role.as_deref() {
        Some("system") => MessageRole::System,
        Some("assistant") => MessageRole::Assistant,
        _ => MessageRole::User,
    };

    let sequence_number = state
        .chat_repository
        .get_next_sequence_number(chat_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let message = state
        .chat_repository
        .create_message(CreateMessageDto {
            chat_id,
            role,
            content: payload.content,
            metadata: None,
            parent_message_id: None,
            sequence_number,
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(MessageResponse::from(message)))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateMessageRequest {
    pub content: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Update a message in a chat
#[utoipa::path(
    put,
    path = "/api/v1/chats/{chat_id}/messages/{id}",
    tag = "Messages",
    security(("bearer_auth" = [])),
    params(
        ("chat_id" = Uuid, Path, description = "Chat identifier"),
        ("id" = Uuid, Path, description = "Message identifier")
    ),
    request_body = UpdateMessageRequest,
    responses(
        (status = 200, description = "Message updated", body = super::MessageResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Chat or message not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn update_message(
    user: AuthenticatedUser,
    state: State<AppState>,
    Path((chat_id, message_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateMessageRequest>,
) -> Result<Json<MessageResponse>, StatusCode> {
    let message = state
        .chat_repository
        .update_message(
            message_id,
            chat_id,
            &user.0.id,
            UpdateMessageDto {
                content: payload.content,
                metadata: payload.metadata,
            },
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(MessageResponse::from(message)))
}

/// Delete a message from a chat
#[utoipa::path(
    delete,
    path = "/api/v1/chats/{chat_id}/messages/{id}",
    tag = "Messages",
    security(("bearer_auth" = [])),
    params(
        ("chat_id" = Uuid, Path, description = "Chat identifier"),
        ("id" = Uuid, Path, description = "Message identifier")
    ),
    responses(
        (status = 204, description = "Message deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Chat or message not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn delete_message(
    user: AuthenticatedUser,
    state: State<AppState>,
    Path((chat_id, message_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    state
        .chat_repository
        .delete_message(message_id, chat_id, &user.0.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

/// Clear all messages from a chat
#[utoipa::path(
    delete,
    path = "/api/v1/chats/{id}/messages",
    tag = "Messages",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Chat identifier")
    ),
    responses(
        (status = 204, description = "All messages deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Chat not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn clear_messages(
    user: AuthenticatedUser,
    state: State<AppState>,
    Path(chat_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    state
        .chat_repository
        .clear_messages(chat_id, &user.0.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}