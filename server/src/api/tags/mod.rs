use crate::{
    AppState,
    db::models::{NewTag, Tag, UpdateTag},
    db::repositories::tag_repository::TTagRepository,
    middleware::auth::AuthenticatedUser,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use serde::Deserialize;
use uuid::Uuid;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateTagRequest {
    pub name: String,
    pub color: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTagRequest {
    pub name: Option<String>,
    pub color: Option<String>,
    pub description: Option<String>,
    pub position: Option<i32>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReorderTagsRequest {
    pub tag_ids: Vec<Uuid>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_tags).post(create_tag))
        .route("/{id}", get(get_tag).put(update_tag).delete(delete_tag))
        .route("/reorder", post(reorder_tags))
}

async fn list_tags(
    user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<Tag>>, StatusCode> {
    let tags = state
        .tag_repository
        .list(&user.0.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(tags))
}

async fn create_tag(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateTagRequest>,
) -> Result<Json<Tag>, StatusCode> {
    let new_tag = NewTag {
        id: Some(Uuid::new_v4()),
        user_id: user.0.id.clone(),
        name: payload.name,
        description: payload.description,
        color: payload.color,
        position: None,
    };

    let tag = state
        .tag_repository
        .create(new_tag)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(tag))
}

async fn get_tag(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Tag>, StatusCode> {
    let tag = state
        .tag_repository
        .get(id, &user.0.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(tag))
}

async fn update_tag(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateTagRequest>,
) -> Result<Json<Tag>, StatusCode> {
    let update = UpdateTag {
        name: payload.name,
        color: payload.color.map(Some),
        description: payload.description.map(Some),
        position: payload.position,
        updated_at: chrono::Utc::now(),
    };

    let tag = state
        .tag_repository
        .update(id, &user.0.id, update)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(tag))
}

async fn delete_tag(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    state
        .tag_repository
        .delete(id, &user.0.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::OK)
}

async fn reorder_tags(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<ReorderTagsRequest>,
) -> Result<StatusCode, StatusCode> {
    for (index, id) in payload.tag_ids.iter().enumerate() {
        let update = UpdateTag {
            name: None,
            color: None,
            description: None,
            position: Some(index as i32),
            updated_at: chrono::Utc::now(),
        };
        
        let _ = state.tag_repository.update(*id, &user.0.id, update).await;
    }

    Ok(StatusCode::OK)
}

