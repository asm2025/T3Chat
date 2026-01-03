use crate::{
    AppState,
    db::models::Assistant,
    db::repositories::agent_repository::TAgentRepository,
    middleware::auth::AuthenticatedUser,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use uuid::Uuid;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_assistants))
        .route("/sync", post(sync_assistants))
        .route("/:id", get(get_assistant))
}

async fn list_assistants(
    user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<Assistant>>, StatusCode> {
    // Reusing AgentRepository which implements TAgentRepository that includes Assistant methods
    let assistants = state
        .agent_repository
        .list_assistants(&user.0.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(assistants))
}

async fn get_assistant(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Assistant>, StatusCode> {
    let assistant = state
        .agent_repository
        .get_assistant(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Verify ownership
    if assistant.user_id != user.0.id {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(Json(assistant))
}

async fn sync_assistants(
    user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<Assistant>>, StatusCode> {
    // For now, sync is a no-op that just returns the current list
    // In the future, this could sync with OpenAI Assistants API
    let assistants = state
        .agent_repository
        .list_assistants(&user.0.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(assistants))
}

