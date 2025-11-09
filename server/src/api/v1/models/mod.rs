use crate::{AppState, db::prelude::*, db::repositories::IAiModelRepository};
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
    pub id: Uuid,
    pub provider: String,
    pub model_id: String,
    pub display_name: String,
    pub description: Option<String>,
    pub context_window: i32,
    pub supports_streaming: bool,
    pub supports_images: bool,
    pub supports_functions: bool,
    pub cost_per_token: Option<rust_decimal::Decimal>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<AiModelModel> for ModelResponse {
    fn from(model: AiModelModel) -> Self {
        Self {
            id: model.id,
            provider: match model.provider {
                AiProvider::OpenAI => "openai".to_string(),
                AiProvider::Anthropic => "anthropic".to_string(),
                AiProvider::Google => "google".to_string(),
                AiProvider::DeepSeek => "deepseek".to_string(),
                AiProvider::Ollama => "ollama".to_string(),
            },
            model_id: model.model_id,
            display_name: model.display_name,
            description: model.description,
            context_window: model.context_window,
            supports_streaming: model.supports_streaming,
            supports_images: model.supports_images,
            supports_functions: model.supports_functions,
            cost_per_token: model.cost_per_token,
            is_active: model.is_active,
            created_at: model.created_at.to_rfc3339(),
            updated_at: model.updated_at.to_rfc3339(),
        }
    }
}

/// List all active AI models
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
    let models = state
        .ai_model_repository
        .list_active()
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
        .get_by_id(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(ModelResponse::from(model)))
}
