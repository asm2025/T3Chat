use axum::extract::Request;
use axum::{extract::State, http::StatusCode, middleware::Next, response::Response};
use tracing::warn;

use crate::AppState;
use crate::middleware::auth::AuthenticatedUser;

pub async fn admin_middleware(
    State(state): State<AppState>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract user from request extensions (set by auth middleware)
    let user = request
        .extensions()
        .get::<AuthenticatedUser>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Check if user has admin role from user_roles table
    let user_repo = crate::db::repositories::UserRepository::new(state.db.clone());
    let has_admin = user_repo.has_role(&user.0.id, "admin").await.map_err(|e| {
        warn!("Failed to check admin role for user {}: {}", user.0.id, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if !has_admin {
        warn!(
            "Non-admin user {} attempted to access admin endpoint",
            user.0.id
        );
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(request).await)
}
