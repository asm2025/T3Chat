use crate::{
    AppState, db::prelude::*, db::repositories::TAiModelRepository,
    middleware::auth::AuthenticatedUser,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub struct ModelResponse {
    pub id: String,
    pub provider: String,
    pub model_id: String,
    pub display_name: String,
    pub description: Option<String>,
    pub context_window: i32,
    pub supports_streaming: bool,
    pub supports_images: bool,
    pub supports_functions: bool,
    pub cost_per_token: Option<f64>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl ModelResponse {
    /// Create from catalog ModelInfo with provider context
    fn from_catalog(provider: &str, info: &crate::ai::types::ModelInfo) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: format!("{}:{}", provider, info.id),
            provider: provider.to_string(),
            model_id: info.id.clone(),
            display_name: info.display_name.clone(),
            description: None,
            context_window: info.context_window as i32,
            supports_streaming: info.supports_streaming,
            supports_images: info.supports_images || info.supports_vision,
            supports_functions: info.supports_functions,
            cost_per_token: info.cost_per_input_token,
            is_active: true,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

impl From<(AiModelModel, String)> for ModelResponse {
    fn from((model, provider_name): (AiModelModel, String)) -> Self {
        Self {
            id: model.id.to_string(),
            provider: provider_name,
            model_id: model.model_id,
            display_name: model.display_name,
            description: model.description,
            context_window: model.context_window,
            supports_streaming: model.supports_streaming.unwrap_or(false),
            supports_images: model.supports_images.unwrap_or(false),
            supports_functions: model.supports_functions.unwrap_or(false),
            cost_per_token: None,
            is_active: model.is_active.unwrap_or(true),
            created_at: model.created_at.to_rfc3339(),
            updated_at: model.updated_at.to_rfc3339(),
        }
    }
}

/// List all active AI models from the model catalog
#[utoipa::path(
    get,
    path = "/api/v1/models",
    tag = "Models",
    responses(
        (status = 200, description = "List of AI models", body = [ModelResponse]),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn list_models(state: State<AppState>) -> Result<Json<Vec<ModelResponse>>, StatusCode> {
    // Return models directly from the model catalog (runtime source of truth)
    let catalog = &state.model_catalog;
    let mut models: Vec<ModelResponse> = Vec::new();

    for (provider, provider_models) in catalog.all() {
        for model_info in provider_models {
            models.push(ModelResponse::from_catalog(provider, model_info));
        }
    }

    // Sort by provider then display_name for consistent ordering
    models.sort_by(|a, b| {
        a.provider
            .cmp(&b.provider)
            .then_with(|| a.display_name.cmp(&b.display_name))
    });

    Ok(Json(models))
}

/// List all AI models (active and inactive)
#[utoipa::path(
    get,
    path = "/api/v1/models/all",
    tag = "Models",
    responses(
        (status = 200, description = "List of all AI models", body = [ModelResponse]),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn list_all_models(
    state: State<AppState>,
) -> Result<Json<Vec<ModelResponse>>, StatusCode> {
    let models = state
        .ai_model_repository
        .list()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(models.into_iter().map(ModelResponse::from).collect()))
}

/// Get a specific AI model by ID
#[utoipa::path(
    get,
    path = "/api/v1/models/{id}",
    tag = "Models",
    params(
        ("id" = Uuid, Path, description = "Model identifier")
    ),
    responses(
        (status = 200, description = "AI model detail", body = ModelResponse),
        (status = 404, description = "Model not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_model(
    state: State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ModelResponse>, StatusCode> {
    let model = state
        .ai_model_repository
        .get(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(ModelResponse::from(model)))
}

/// List models enabled for the authenticated user
#[utoipa::path(
    get,
    path = "/api/v1/models/my",
    tag = "Models",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of user's enabled models", body = [ModelResponse]),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn list_my_models(
    _user: AuthenticatedUser,
    state: State<AppState>,
) -> Result<Json<Vec<ModelResponse>>, StatusCode> {
    // For now, return all catalog models (user-specific filtering can be added later)
    // Return models directly from the model catalog (runtime source of truth)
    let catalog = &state.model_catalog;
    let mut models: Vec<ModelResponse> = Vec::new();

    for (provider, provider_models) in catalog.all() {
        for model_info in provider_models {
            models.push(ModelResponse::from_catalog(provider, model_info));
        }
    }

    // Sort by provider then display_name for consistent ordering
    models.sort_by(|a, b| {
        a.provider
            .cmp(&b.provider)
            .then_with(|| a.display_name.cmp(&b.display_name))
    });

    Ok(Json(models))
}

/// Enable a model for the authenticated user
#[utoipa::path(
    post,
    path = "/api/v1/models/{id}/enable",
    tag = "Models",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Model identifier")
    ),
    responses(
        (status = 204, description = "Model enabled"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Model not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn enable_model(
    user: AuthenticatedUser,
    state: State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    // Verify model exists
    let _model = state
        .ai_model_repository
        .get(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    state
        .ai_model_repository
        .enable_for_user(&user.0.id, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

/// Disable a model for the authenticated user
#[utoipa::path(
    post,
    path = "/api/v1/models/{id}/disable",
    tag = "Models",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Model identifier")
    ),
    responses(
        (status = 204, description = "Model disabled"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Model not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn disable_model(
    user: AuthenticatedUser,
    state: State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    // Verify model exists
    let _model = state
        .ai_model_repository
        .get(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    state
        .ai_model_repository
        .disable_for_user(&user.0.id, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}
