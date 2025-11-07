use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use uuid::Uuid;
use crate::{AppState, db::prelude::*, middleware::auth::AuthenticatedUser, api::chats::MessageResponse};

#[derive(Debug, Deserialize)]
pub struct CreateMessageRequest {
    pub content: String,
    pub role: Option<String>,
}

pub async fn get_messages(
    user: AuthenticatedUser,
    state: State<AppState>,
    Path(chat_id): Path<Uuid>,
) -> Result<Json<Vec<MessageResponse>>, StatusCode> {
    let messages = state
        .message_repository
        .list_by_chat(chat_id, &user.0.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(messages.into_iter().map(MessageResponse::from).collect()))
}

pub async fn create_message(
    user: AuthenticatedUser,
    state: State<AppState>,
    Path(chat_id): Path<Uuid>,
    Json(payload): Json<CreateMessageRequest>,
) -> Result<Json<MessageResponse>, StatusCode> {
    // Verify chat belongs to user
    let _chat = state
        .chat_repository
        .get_by_id(chat_id, &user.0.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let role = match payload.role.as_deref() {
        Some("system") => MessageRole::System,
        Some("assistant") => MessageRole::Assistant,
        _ => MessageRole::User,
    };

    let sequence_number = state
        .message_repository
        .get_next_sequence_number(chat_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let message = state
        .message_repository
        .create(CreateMessageDto {
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

