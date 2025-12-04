use crate::AppState;
use crate::ai::types::ModelInfo;
use axum::{extract::State, response::Json};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct ProviderModelsResponse {
    pub providers: HashMap<String, Vec<CatalogModel>>,
}

#[derive(Debug, Serialize)]
pub struct CatalogModel {
    pub id: String,
    pub display_name: String,
    pub context_window: u32,
    pub max_output_tokens: Option<u32>,
    pub supports_streaming: bool,
    pub supports_images: bool,
    pub supports_functions: bool,
    pub supports_vision: bool,
    pub cost_per_input_token: Option<f64>,
    pub cost_per_output_token: Option<f64>,
}

pub async fn list_models(state: State<AppState>) -> Json<ProviderModelsResponse> {
    let mut providers = HashMap::new();

    for (key, models) in state.model_catalog.all().iter() {
        providers.insert(key.clone(), models.iter().map(CatalogModel::from).collect());
    }

    Json(ProviderModelsResponse { providers })
}

impl From<&ModelInfo> for CatalogModel {
    fn from(info: &ModelInfo) -> Self {
        Self {
            id: info.id.clone(),
            display_name: info.display_name.clone(),
            context_window: info.context_window,
            max_output_tokens: info.max_output_tokens,
            supports_streaming: info.supports_streaming,
            supports_images: info.supports_images,
            supports_functions: info.supports_functions,
            supports_vision: info.supports_vision,
            cost_per_input_token: info.cost_per_input_token,
            cost_per_output_token: info.cost_per_output_token,
        }
    }
}
