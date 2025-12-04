use crate::ai::providers::openrouter::OpenRouterClient;
use crate::ai::providers::{
    AIProvider, anthropic::AnthropicProvider, google::GoogleProvider, openai::OpenAIProvider,
};
use crate::ai::types::ModelInfo;
use crate::config::t3chat::{
    DetailedModelConfig, ProviderConfig, ProviderModelEntry, T3ChatConfig,
};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tracing::{info, warn};

pub struct ModelCatalog {
    provider_models: HashMap<String, Vec<ModelInfo>>,
}

impl ModelCatalog {
    pub async fn build(config: Arc<T3ChatConfig>) -> Result<Self, ModelCatalogError> {
        let mut provider_models: HashMap<String, Vec<ModelInfo>> = HashMap::new();
        let entries = config.providers.resolved_entries();

        if entries.is_empty() {
            return Err(ModelCatalogError::NoProvidersConfigured);
        }

        for (key, provider_config) in entries {
            if !provider_config.enabled {
                continue;
            }

            match collect_models_for_provider(&key, &provider_config).await {
                Ok(models) if !models.is_empty() => {
                    info!(provider = %key, model_count = models.len(), "model catalog entry created");
                    provider_models.insert(key, models);
                }
                Ok(_) => {
                    warn!(provider = %key, "provider has no models after fetch/default merge");
                }
                Err(err) => {
                    warn!(provider = %key, error = %err, "failed to collect models for provider");
                }
            }
        }

        if provider_models.is_empty() {
            return Err(ModelCatalogError::NoActiveModels);
        }

        Ok(Self { provider_models })
    }

    pub fn all(&self) -> &HashMap<String, Vec<ModelInfo>> {
        &self.provider_models
    }

    pub fn for_provider(&self, key: &str) -> Option<&[ModelInfo]> {
        self.provider_models
            .get(key)
            .map(|models| models.as_slice())
    }

    pub fn find(&self, key: &str, model_id: &str) -> Option<&ModelInfo> {
        self.provider_models
            .get(key)
            .and_then(|models| models.iter().find(|info| info.id == model_id))
    }
}

#[derive(Debug, Error)]
pub enum ModelCatalogError {
    #[error("no providers configured in t3chat.yaml")]
    NoProvidersConfigured,
    #[error("no active providers exposed any models")]
    NoActiveModels,
    #[error("provider {0} requires an api_key to fetch models dynamically")]
    MissingApiKey(String),
    #[error("provider {0} fetch failed: {1}")]
    FetchFailed(String, String),
}

async fn collect_models_for_provider(
    provider_key: &str,
    config: &ProviderConfig,
) -> Result<Vec<ModelInfo>, ModelCatalogError> {
    let mut models = Vec::new();

    if config.models.fetch {
        models = fetch_remote_models(provider_key, config).await?;
    }

    if models.is_empty() {
        models = config
            .models
            .default
            .iter()
            .map(|entry| to_model_info(provider_key, entry))
            .collect();
    }

    Ok(models)
}

async fn fetch_remote_models(
    provider_key: &str,
    config: &ProviderConfig,
) -> Result<Vec<ModelInfo>, ModelCatalogError> {
    match provider_key {
        "openai" => {
            let provider = OpenAIProvider::new(String::new());
            Ok(<OpenAIProvider as AIProvider>::list_models(&provider))
        }
        "anthropic" => {
            let provider = AnthropicProvider::new(String::new());
            Ok(<AnthropicProvider as AIProvider>::list_models(&provider))
        }
        "google" => {
            let provider = GoogleProvider::new(String::new());
            Ok(<GoogleProvider as AIProvider>::list_models(&provider))
        }
        "openrouter" => {
            let api_key = config
                .api_key
                .clone()
                .ok_or_else(|| ModelCatalogError::MissingApiKey(provider_key.to_string()))?;
            let client = OpenRouterClient::new(api_key, config.base_url.clone());
            client.list_models().await.map_err(|err| {
                ModelCatalogError::FetchFailed(provider_key.to_string(), err.to_string())
            })
        }
        _ => {
            warn!(
                provider = %provider_key,
                "dynamic fetch requested but no provider client implemented; falling back to defaults"
            );
            Ok(Vec::new())
        }
    }
}

fn to_model_info(provider_key: &str, entry: &ProviderModelEntry) -> ModelInfo {
    let detail: DetailedModelConfig = entry.as_detailed();
    ModelInfo {
        id: detail.id.clone(),
        display_name: detail
            .label
            .clone()
            .unwrap_or_else(|| format!("{} ({provider_key})", detail.id)),
        context_window: detail.context_window.unwrap_or(0),
        max_output_tokens: detail.max_output_tokens,
        supports_streaming: detail.supports_streaming.unwrap_or(true),
        supports_images: detail.supports_images.unwrap_or(false),
        supports_functions: detail.supports_functions.unwrap_or(false),
        supports_vision: detail.supports_vision.unwrap_or(false),
        cost_per_input_token: detail.cost_per_input_token,
        cost_per_output_token: detail.cost_per_output_token,
    }
}
