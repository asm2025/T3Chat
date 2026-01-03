use axum::{
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppState;
use crate::db::models::ai_model::AiModel;
use crate::db::repositories::{TAiModelRepository, TAiProviderRepository};
use crate::middleware::auth::AuthenticatedUser;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListModelsQuery {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_limit")]
    pub limit: u64,
    #[serde(alias = "provider_id")]
    pub provider_id: Option<Uuid>,
    pub disabled: Option<bool>,
}

fn default_page() -> u64 {
    1
}

fn default_limit() -> u64 {
    20
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelResponse {
    pub id: Uuid,
    pub provider: String,
    pub model_id: String,
    pub display_name: String,
    pub description: Option<String>,
    pub context_window: i32,
    pub max_output_tokens: Option<i32>,
    pub supports_streaming: Option<bool>,
    pub supports_images: Option<bool>,
    pub supports_functions: Option<bool>,
    pub supports_vision: Option<bool>,
    pub cost_per_input_token: Option<rust_decimal::Decimal>,
    pub cost_per_output_token: Option<rust_decimal::Decimal>,
    pub is_active: Option<bool>,
    pub deprecated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub provider_id: Option<Uuid>,
    pub disabled: bool,
    pub is_paid: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<(AiModel, String)> for ModelResponse {
    fn from((model, provider_name): (AiModel, String)) -> Self {
        Self {
            id: model.id,
            provider: provider_name,
            model_id: model.model_id,
            display_name: model.display_name,
            description: model.description,
            context_window: model.context_window,
            max_output_tokens: model.max_output_tokens,
            supports_streaming: model.supports_streaming,
            supports_images: model.supports_images,
            supports_functions: model.supports_functions,
            supports_vision: model.supports_vision,
            cost_per_input_token: model.cost_per_input_token,
            cost_per_output_token: model.cost_per_output_token,
            is_active: model.is_active,
            deprecated_at: model.deprecated_at,
            provider_id: model.provider_id,
            disabled: model.disabled,
            is_paid: model.is_paid,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateModelRequest {
    pub provider: String,
    pub model_id: String,
    pub display_name: String,
    pub description: Option<String>,
    pub context_window: i32,
    pub max_output_tokens: Option<i32>,
    pub supports_streaming: Option<bool>,
    pub supports_images: Option<bool>,
    pub supports_functions: Option<bool>,
    pub supports_vision: Option<bool>,
    pub cost_per_input_token: Option<rust_decimal::Decimal>,
    pub cost_per_output_token: Option<rust_decimal::Decimal>,
    pub provider_id: Option<Uuid>,
    pub disabled: Option<bool>,
    pub is_paid: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateModelRequest {
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub context_window: Option<i32>,
    pub max_output_tokens: Option<i32>,
    pub supports_streaming: Option<bool>,
    pub supports_images: Option<bool>,
    pub supports_functions: Option<bool>,
    pub supports_vision: Option<bool>,
    pub cost_per_input_token: Option<rust_decimal::Decimal>,
    pub cost_per_output_token: Option<rust_decimal::Decimal>,
    pub is_active: Option<bool>,
    pub deprecated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub disabled: Option<bool>,
    pub is_paid: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListModelsResponse {
    pub data: Vec<ModelResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

pub async fn list_models(
    State(state): State<AppState>,
    Query(params): Query<ListModelsQuery>,
    _user: AuthenticatedUser,
) -> Result<Json<ListModelsResponse>, StatusCode> {
    // For now, use list() - TODO: Add filtering
    let models = state
        .ai_model_repository
        .list()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Apply pagination manually for now
    let page = params.page;
    let page_size = params.limit;
    let start = ((page - 1) * page_size) as usize;
    let end = (start + page_size as usize).min(models.len());
    let paginated = models[start..end].to_vec();

    Ok(Json(ListModelsResponse {
        data: paginated.into_iter().map(ModelResponse::from).collect(),
        total: models.len() as u64,
        page: page,
        page_size: page_size,
    }))
}

pub async fn get_model(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _user: AuthenticatedUser,
) -> Result<Json<ModelResponse>, StatusCode> {
    let model = state
        .ai_model_repository
        .get(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(ModelResponse::from(model)))
}

pub async fn create_model(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Json(req): Json<CreateModelRequest>,
) -> Result<Json<ModelResponse>, StatusCode> {
    use crate::db::models::ai_model::NewAiModel;

    let provider_id = if let Some(id) = req.provider_id {
        id
    } else {
        let provider = state
            .ai_provider_repository
            .get_by_provider_id(&req.provider)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::BAD_REQUEST)?;
        provider.id
    };

    let new_model = NewAiModel {
        id: None,
        // provider: req.provider, // REMOVED
        model_id: req.model_id,
        display_name: req.display_name,
        description: req.description,
        context_window: req.context_window,
        max_output_tokens: req.max_output_tokens,
        supports_streaming: req.supports_streaming,
        supports_images: req.supports_images,
        supports_functions: req.supports_functions,
        supports_vision: req.supports_vision,
        cost_per_input_token: req.cost_per_input_token,
        cost_per_output_token: req.cost_per_output_token,
        provider_id: Some(provider_id),
        disabled: req.disabled.unwrap_or(false),
        is_paid: req.is_paid.unwrap_or(true),
    };

    let model = state
        .ai_model_repository
        .create(new_model)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ModelResponse::from(model)))
}

pub async fn update_model(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _user: AuthenticatedUser,
    Json(req): Json<UpdateModelRequest>,
) -> Result<Json<ModelResponse>, StatusCode> {
    use crate::db::models::ai_model::UpdateAiModel;

    let update_model = UpdateAiModel {
        display_name: req.display_name,
        description: req.description.map(Some),
        context_window: req.context_window,
        max_output_tokens: req.max_output_tokens.map(Some),
        supports_streaming: req.supports_streaming,
        supports_images: req.supports_images,
        supports_functions: req.supports_functions,
        supports_vision: req.supports_vision,
        cost_per_input_token: req.cost_per_input_token.map(Some),
        cost_per_output_token: req.cost_per_output_token.map(Some),
        is_active: req.is_active,
        deprecated_at: None, // Don't allow updating deprecated_at via this endpoint
        provider_id: None,   // Don't allow updating provider_id via this endpoint
        disabled: req.disabled,
        is_paid: req.is_paid,
        updated_at: chrono::Utc::now(),
    };

    let model = state
        .ai_model_repository
        .update(id, update_model)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ModelResponse::from(model)))
}

pub async fn delete_model(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _user: AuthenticatedUser,
) -> Result<StatusCode, StatusCode> {
    state
        .ai_model_repository
        .delete(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn enable_model(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _user: AuthenticatedUser,
) -> Result<Json<ModelResponse>, StatusCode> {
    state
        .ai_model_repository
        .enable(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    get_model(State(state), Path(id), _user).await
}

pub async fn disable_model(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _user: AuthenticatedUser,
) -> Result<Json<ModelResponse>, StatusCode> {
    state
        .ai_model_repository
        .disable(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    get_model(State(state), Path(id), _user).await
}

pub async fn deprecate_model(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _user: AuthenticatedUser,
) -> Result<Json<ModelResponse>, StatusCode> {
    use diesel_async::RunQueryDsl;

    let mut conn = state
        .db
        .get()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    diesel::sql_query("UPDATE ai_models SET deprecated_at = NOW() WHERE id = $1")
        .bind::<diesel::sql_types::Uuid, _>(&id)
        .execute(&mut conn)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    get_model(State(state), Path(id), _user).await
}

// POST /api/admin/models/scan
pub async fn scan_models(
    _state: State<AppState>,
    _user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Placeholder - returns 501 Not Implemented
    Err(StatusCode::NOT_IMPLEMENTED)
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_models).post(create_model))
        .route(
            "/{id}",
            get(get_model).put(update_model).delete(delete_model),
        )
        .route("/{id}/enable", post(enable_model))
        .route("/{id}/disable", post(disable_model))
        .route("/{id}/deprecate", post(deprecate_model))
        .route("/scan", post(scan_models))
}
