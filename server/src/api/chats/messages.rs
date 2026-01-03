use super::MessageResponse;
use crate::{
    AppState, db::prelude::*, db::repositories::chat_repository::TChatRepository,
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
#[serde(rename_all = "camelCase")]
pub struct CreateMessageRequest {
    pub content: String,
    pub role: Option<String>,
}

/// Get all messages for a chat
#[utoipa::path(
    get,
    path = "/api/chats/{id}/messages",
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
    // Resolve `chat_id` which might be either the internal UUID or the external chat_id (TEXT)
    let chat = state
        .chat_repository
        .get(chat_id, &user.0.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let messages = state
        .chat_repository
        .list_messages(chat.id, &user.0.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(
        messages.into_iter().map(MessageResponse::from).collect(),
    ))
}

/// Create a new message in a chat
#[utoipa::path(
    post,
    path = "/api/chats/{id}/messages",
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
    tracing::info!(
        "Create message request: chat_id={}, payload={:?}",
        chat_id,
        payload
    );

    // Verify chat belongs to user
    let chat = state
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
        .get_next_sequence_number(chat.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let message = state
        .chat_repository
        .create_message(CreateMessageDto {
            chat_id: chat.id,
            role,
            content: payload.content,
            metadata: None,
            parent_message_id: None,
            sequence_number,
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Index in MeiliSearch (non-blocking)
    let meili = state.meilisearch.clone();
    let message_doc = crate::utils::meilisearch::MeiliDocument {
        id: format!("message:{}", message.id),
        user_id: user.0.id.clone(),
        chat_id: message.chat_id.to_string(),
        r#type: "message".to_string(),
        title: None,
        text: message.text.clone(),
        role: Some(message.role.clone()),
        model: message.model.clone(),
        endpoint: None,
        created_at: message.created_at.to_rfc3339(),
        updated_at: message.updated_at.to_rfc3339(),
    };
    tokio::spawn(async move {
        if let Err(e) = meili.index_message(message_doc).await {
            tracing::warn!("Failed to index message in MeiliSearch: {}", e);
        }
    });

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
    path = "/api/chats/{chat_id}/messages/{id}",
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
    // Resolve `chat_id` which might be either the internal UUID or the external chat_id (TEXT)
    let chat = state
        .chat_repository
        .get(chat_id, &user.0.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let message = state
        .chat_repository
        .update_message(
            message_id,
            chat.id,
            &user.0.id,
            UpdateMessageDto {
                content: payload.content,
                metadata: payload.metadata,
            },
        )
        .await
        .map_err(|e| {
            let err_msg = e.to_string().to_lowercase();
            if err_msg.contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    // Update index in MeiliSearch (non-blocking)
    let meili = state.meilisearch.clone();
    let message_doc = crate::utils::meilisearch::MeiliDocument {
        id: format!("message:{}", message.id),
        user_id: user.0.id.clone(),
        chat_id: message.chat_id.to_string(),
        r#type: "message".to_string(),
        title: None,
        text: message.text.clone(),
        role: Some(message.role.clone()),
        model: message.model.clone(),
        endpoint: None,
        created_at: message.created_at.to_rfc3339(),
        updated_at: message.updated_at.to_rfc3339(),
    };
    tokio::spawn(async move {
        if let Err(e) = meili.index_message(message_doc).await {
            tracing::warn!("Failed to update message in MeiliSearch: {}", e);
        }
    });

    Ok(Json(MessageResponse::from(message)))
}

/// Delete a message from a chat
#[utoipa::path(
    delete,
    path = "/api/chats/{chat_id}/messages/{id}",
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
    // Resolve `chat_id` which might be either the internal UUID or the external chat_id (TEXT)
    let chat = state
        .chat_repository
        .get(chat_id, &user.0.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    state
        .chat_repository
        .delete_message(message_id, chat.id, &user.0.id)
        .await
        .map_err(|e| {
            let err_msg = e.to_string().to_lowercase();
            if err_msg.contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    // Delete from MeiliSearch (non-blocking)
    let meili = state.meilisearch.clone();
    let message_id_str = message_id.to_string();
    tokio::spawn(async move {
        if let Err(e) = meili.delete_message(&message_id_str).await {
            tracing::warn!("Failed to delete message from MeiliSearch: {}", e);
        }
    });

    Ok(StatusCode::NO_CONTENT)
}

/// Clear all messages from a chat
#[utoipa::path(
    delete,
    path = "/api/chats/{id}/messages",
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
    // Resolve `chat_id` which might be either the internal UUID or the external chat_id (TEXT)
    let chat = state
        .chat_repository
        .get(chat_id, &user.0.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    state
        .chat_repository
        .clear_messages(chat.id, &user.0.id)
        .await
        .map_err(|e| {
            let err_msg = e.to_string().to_lowercase();
            if err_msg.contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    Ok(StatusCode::NO_CONTENT)
}
