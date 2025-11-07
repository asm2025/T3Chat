use crate::db::schema::UpdateUserDto;
use crate::{AppState, api::UserResponse, middleware::auth::AuthenticatedUser};
use axum::{extract::State, http::StatusCode, response::Json};
use emixdb::repositories::IRepository;
use serde::Deserialize;

/// Get current user profile
pub async fn profile(user: AuthenticatedUser) -> Result<Json<UserResponse>, StatusCode> {
    Ok(Json(UserResponse::from(user.0)))
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    // Since frontend always sends display_name (either string or null),
    // we can use Option<String> directly. When null is sent, it becomes None.
    // When a value is sent, it becomes Some(value).
    // We use #[serde(default)] so missing fields don't cause errors.
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub image_url: Option<String>,
}

/// Update current user profile
pub async fn update_profile(
    user: AuthenticatedUser,
    state: State<AppState>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, StatusCode> {
    // Debug: log what we received
    tracing::debug!(
        "UpdateUserRequest received: display_name={:?}, image_url={:?}",
        payload.display_name,
        payload.image_url
    );

    // Convert Option<String> to Option<Option<String>> for UpdateUserDto:
    // Since frontend always sends display_name, if it's Some(value), wrap it
    // If it's None, it means null was sent, so we want Some(None)
    // But wait - if the field is missing, #[serde(default)] makes it None
    // So we need to distinguish: was it sent as null, or was it missing?
    // Since frontend ALWAYS sends display_name, None here means null was sent
    let update_dto = UpdateUserDto {
        display_name: Some(payload.display_name), // Always wrap since frontend always sends it
        image_url: payload.image_url.map(|v| Some(v)), // Only wrap if provided
    };

    tracing::debug!(
        "UpdateUserDto created: display_name={:?}, image_url={:?}",
        update_dto.display_name,
        update_dto.image_url
    );

    let updated_user = state
        .user_repository
        .update(user.0.id.clone(), update_dto)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update user: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    tracing::debug!(
        "User updated successfully: display_name={:?}",
        updated_user.display_name
    );

    Ok(Json(UserResponse::from(updated_user)))
}
