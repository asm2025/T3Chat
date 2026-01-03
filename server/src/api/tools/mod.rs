use crate::{
    AppState,
    db::models::Tool,
    db::repositories::tool_repository::TToolRepository,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ListToolsQuery {
    pub is_active: Option<bool>,
    pub tool_type: Option<String>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_tools))
        .route("/:id", get(get_tool))
}

async fn list_tools(
    State(state): State<AppState>,
    Query(params): Query<ListToolsQuery>,
) -> Result<Json<Vec<Tool>>, StatusCode> {
    let mut tools = state
        .tool_repository
        .list_active()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Filter by tool_type if provided
    if let Some(tool_type) = params.tool_type {
        tools.retain(|t| t.tool_type == tool_type);
    }

    // Filter by is_active if explicitly false (list_active already filters for active)
    if let Some(false) = params.is_active {
        tools.retain(|t| t.is_active == Some(false));
    }

    Ok(Json(tools))
}

async fn get_tool(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Tool>, StatusCode> {
    let tool = state
        .tool_repository
        .get(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(tool))
}

