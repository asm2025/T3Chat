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
use crate::db::dto::Pagination;
use crate::db::models::ai_provider::{AiProvider, NewAiProvider, UpdateAiProvider};
use crate::db::repositories::{AiProviderRepository, TAiProviderRepository};
use crate::middleware::auth::AuthenticatedUser;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListProvidersQuery {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_limit")]
    pub limit: u64,
}

fn default_page() -> u64 {
    1
}

fn default_limit() -> u64 {
    20
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderResponse {
    pub id: Uuid,
    pub provider_id: String,
    pub display_name: String,
    pub description: Option<String>,
    pub base_url: Option<String>,
    pub website_url: Option<String>,
    pub documentation_url: Option<String>,
    pub disabled: bool,
    pub is_active: bool,
    pub requires_api_key: bool,
    pub supports_streaming: bool,
    pub supports_images: bool,
    pub supports_functions: bool,
    pub supports_vision: bool,
    pub metadata: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<AiProvider> for ProviderResponse {
    fn from(provider: AiProvider) -> Self {
        Self {
            id: provider.id,
            provider_id: provider.provider_id,
            display_name: provider.display_name,
            description: provider.description,
            base_url: provider.base_url,
            website_url: provider.website_url,
            documentation_url: provider.documentation_url,
            disabled: provider.disabled,
            is_active: provider.is_active,
            requires_api_key: provider.requires_api_key,
            supports_streaming: provider.supports_streaming,
            supports_images: provider.supports_images,
            supports_functions: provider.supports_functions,
            supports_vision: provider.supports_vision,
            metadata: provider.metadata,
            created_at: provider.created_at,
            updated_at: provider.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProviderRequest {
    pub provider_id: String,
    pub display_name: String,
    pub description: Option<String>,
    pub base_url: Option<String>,
    pub website_url: Option<String>,
    pub documentation_url: Option<String>,
    pub disabled: Option<bool>,
    pub is_active: Option<bool>,
    pub requires_api_key: Option<bool>,
    pub supports_streaming: Option<bool>,
    pub supports_images: Option<bool>,
    pub supports_functions: Option<bool>,
    pub supports_vision: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProviderRequest {
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub base_url: Option<String>,
    pub website_url: Option<String>,
    pub documentation_url: Option<String>,
    pub disabled: Option<bool>,
    pub is_active: Option<bool>,
    pub requires_api_key: Option<bool>,
    pub supports_streaming: Option<bool>,
    pub supports_images: Option<bool>,
    pub supports_functions: Option<bool>,
    pub supports_vision: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListProvidersResponse {
    pub data: Vec<ProviderResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

pub async fn list_providers(
    State(state): State<AppState>,
    Query(params): Query<ListProvidersQuery>,
    _user: AuthenticatedUser,
) -> Result<Json<ListProvidersResponse>, StatusCode> {
    let provider_repo = AiProviderRepository::new(state.db.clone());

    let pagination = Pagination {
        page: params.page,
        page_size: params.limit,
    };

    let result = provider_repo
        .list(Some(pagination))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ListProvidersResponse {
        data: result
            .data
            .into_iter()
            .map(ProviderResponse::from)
            .collect(),
        total: result.total,
        page: params.page,
        page_size: params.limit,
    }))
}

pub async fn get_provider(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _user: AuthenticatedUser,
) -> Result<Json<ProviderResponse>, StatusCode> {
    let provider_repo = AiProviderRepository::new(state.db.clone());

    let provider = provider_repo
        .get(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(ProviderResponse::from(provider)))
}

pub async fn create_provider(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Json(req): Json<CreateProviderRequest>,
) -> Result<Json<ProviderResponse>, StatusCode> {
    let provider_repo = AiProviderRepository::new(state.db.clone());

    let new_provider = NewAiProvider {
        id: None,
        provider_id: req.provider_id,
        display_name: req.display_name,
        description: req.description,
        base_url: req.base_url,
        website_url: req.website_url,
        documentation_url: req.documentation_url,
        disabled: req.disabled.unwrap_or(false),
        is_active: req.is_active.unwrap_or(true),
        requires_api_key: req.requires_api_key.unwrap_or(true),
        supports_streaming: req.supports_streaming.unwrap_or(true),
        supports_images: req.supports_images.unwrap_or(false),
        supports_functions: req.supports_functions.unwrap_or(false),
        supports_vision: req.supports_vision.unwrap_or(false),
        metadata: req.metadata,
    };

    let provider = provider_repo
        .create(new_provider)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ProviderResponse::from(provider)))
}

pub async fn update_provider(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _user: AuthenticatedUser,
    Json(req): Json<UpdateProviderRequest>,
) -> Result<Json<ProviderResponse>, StatusCode> {
    let provider_repo = AiProviderRepository::new(state.db.clone());

    let update_provider = UpdateAiProvider {
        display_name: req.display_name,
        description: req.description.map(Some),
        base_url: req.base_url.map(Some),
        website_url: req.website_url.map(Some),
        documentation_url: req.documentation_url.map(Some),
        disabled: req.disabled,
        is_active: req.is_active,
        requires_api_key: req.requires_api_key,
        supports_streaming: req.supports_streaming,
        supports_images: req.supports_images,
        supports_functions: req.supports_functions,
        supports_vision: req.supports_vision,
        metadata: req.metadata.map(Some),
        updated_at: chrono::Utc::now(),
    };

    let provider = provider_repo
        .update(id, update_provider)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ProviderResponse::from(provider)))
}

pub async fn delete_provider(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _user: AuthenticatedUser,
) -> Result<StatusCode, StatusCode> {
    let provider_repo = AiProviderRepository::new(state.db.clone());

    provider_repo
        .delete(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn enable_provider(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _user: AuthenticatedUser,
) -> Result<Json<ProviderResponse>, StatusCode> {
    let provider_repo = AiProviderRepository::new(state.db.clone());

    provider_repo
        .enable(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    get_provider(State(state), Path(id), _user).await
}

pub async fn disable_provider(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _user: AuthenticatedUser,
) -> Result<Json<ProviderResponse>, StatusCode> {
    let provider_repo = AiProviderRepository::new(state.db.clone());

    provider_repo
        .disable(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    get_provider(State(state), Path(id), _user).await
}

// POST /api/admin/providers/{id}/scan
pub async fn scan_provider(
    _state: State<AppState>,
    _path: Path<Uuid>,
    _user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Placeholder - returns 501 Not Implemented
    Err(StatusCode::NOT_IMPLEMENTED)
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_providers).post(create_provider))
        .route(
            "/{id}",
            get(get_provider)
                .put(update_provider)
                .delete(delete_provider),
        )
        .route("/{id}/enable", post(enable_provider))
        .route("/{id}/disable", post(disable_provider))
        .route("/{id}/scan", post(scan_provider))
}
