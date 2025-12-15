use crate::ai::providers::openrouter::OpenRouterClient;
use crate::ai::providers::chatllm::ChatLLMProvider;
use crate::ai::providers::{
    AIProvider, anthropic::AnthropicProvider, google::GoogleProvider, openai::OpenAIProvider,
};
use crate::ai::types::ModelInfo;
use crate::config::app_config::{DerivedAppConfig, DerivedEndpoint};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tracing::{info, warn};

#[derive(Debug)]
pub struct ModelCatalog {
    provider_models: HashMap<String, Vec<ModelInfo>>,
}

impl ModelCatalog {
    pub async fn build(
        config: Arc<DerivedAppConfig>,
        allow_empty: bool,
    ) -> Result<Self, ModelCatalogError> {
        let mut provider_models: HashMap<String, Vec<ModelInfo>> = HashMap::new();
        
        if config.endpoints.is_empty() {
            if allow_empty {
                warn!("No endpoints configured; model catalog will be empty");
                return Ok(Self { provider_models });
            }
            return Err(ModelCatalogError::NoProvidersConfigured);
        }

        for endpoint in &config.endpoints {
            match collect_models_for_endpoint(endpoint).await {
                Ok(models) if !models.is_empty() => {
                    info!(provider = %endpoint.provider, model_count = models.len(), "model catalog entry created");
                    provider_models.insert(endpoint.provider.clone(), models);
                }
                Ok(_) => {
                    warn!(provider = %endpoint.provider, "provider has no models after fetch/default merge");
                }
                Err(err) => {
                    warn!(provider = %endpoint.provider, error = %err, "failed to collect models for provider");
                }
            }
        }

        if provider_models.is_empty() {
            if allow_empty {
                warn!("No providers exposed models; continuing with empty model catalog");
                return Ok(Self { provider_models });
            }
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
    #[error("no providers configured")]
    NoProvidersConfigured,
    #[error("no active providers exposed any models")]
    NoActiveModels,
    #[error("provider {0} requires an api_key to fetch models dynamically")]
    MissingApiKey(String),
    #[error("provider {0} fetch failed: {1}")]
    FetchFailed(String, String),
}

async fn collect_models_for_endpoint(
    endpoint: &DerivedEndpoint,
) -> Result<Vec<ModelInfo>, ModelCatalogError> {
    let mut models = Vec::new();

    if endpoint.models.fetch {
        models = fetch_remote_models(endpoint).await?;
    }

    // Always merge defaults if provided, or if fetch yielded nothing and we have defaults
    // The plan says "Otherwise fall back to configured models.default or an internal static list."
    // But LibreChat usually merges.
    // T3Chat logic: if models.is_empty(), use defaults.
    // Also, if fetch is false, we start with empty.
    
    // If we have defaults in config, add them (deduplicating or appending?)
    // T3Chat used to replace if empty.
    
    if models.is_empty() {
        for model_id in &endpoint.models.default {
             // Create basic ModelInfo from ID
             // We might want to look up details if known, but for now basic.
             // We can use the provider to get details if it knows it statically.
             let info = get_static_model_info(&endpoint.provider, model_id);
             models.push(info);
        }
    }
    
    // Filter not_allowed?
    if !endpoint.models.not_allowed.is_empty() {
        models.retain(|m| !endpoint.models.not_allowed.contains(&m.id));
    }

    Ok(models)
}

fn get_static_model_info(provider: &str, model_id: &str) -> ModelInfo {
     // Try to get detailed info from the provider struct if possible, otherwise generic
     match provider {
         "chatllm" => {
             // For ChatLLM, `default` means "let provider route automatically" (omit model field).
             // Show a friendly label in the UI.
             if model_id == "default" {
                 return ModelInfo {
                     id: "default".to_string(),
                     display_name: "ChatLLM".to_string(),
                     context_window: 128000,
                     max_output_tokens: None,
                     supports_streaming: true,
                     supports_images: false,
                     supports_functions: false,
                     supports_vision: false,
                     cost_per_input_token: None,
                     cost_per_output_token: None,
                 };
             }
         }
         "openai" => {
             let p = OpenAIProvider::new(String::new());
             if let Some(info) = p.get_model_info(model_id) {
                 return info;
             }
         }
         "anthropic" => {
             let p = AnthropicProvider::new(String::new());
             if let Some(info) = p.get_model_info(model_id) {
                 return info;
             }
         }
         "google" => {
             let p = GoogleProvider::new(String::new());
             if let Some(info) = p.get_model_info(model_id) {
                 return info;
             }
         }
         _ => {}
     }

    ModelInfo {
        id: model_id.to_string(),
        display_name: model_id.to_string(), // Could format nicely?
        context_window: 0,
        max_output_tokens: None,
        supports_streaming: true, // Assume yes?
        supports_images: false,
        supports_functions: false,
        supports_vision: false,
        cost_per_input_token: None,
        cost_per_output_token: None,
    }
}

async fn fetch_remote_models(
    endpoint: &DerivedEndpoint,
) -> Result<Vec<ModelInfo>, ModelCatalogError> {
    let provider_key = &endpoint.provider;
    
    match provider_key.as_str() {
        "openai" => {
            // OpenAIProvider::list_models is static currently, so fetching "remote" means static list
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
            let api_key = endpoint
                .api_key
                .clone()
                .ok_or_else(|| ModelCatalogError::MissingApiKey(provider_key.to_string()))?;
            let client = OpenRouterClient::new(api_key, endpoint.base_url.clone());
            client.list_models().await.map_err(|err| {
                ModelCatalogError::FetchFailed(provider_key.to_string(), err.to_string())
            })
        }
        "chatllm" | "ChatLLM" => {
             let api_key = endpoint
                .api_key
                .clone()
                .ok_or_else(|| ModelCatalogError::MissingApiKey(provider_key.to_string()))?;
            let provider = ChatLLMProvider::new(api_key, endpoint.base_url.clone());
            provider.fetch_models().await.map_err(|err| {
                ModelCatalogError::FetchFailed(provider_key.to_string(), err.to_string())
            })
        }
        _ => {
            // For custom providers, we don't have a generic fetch yet unless we implement "OpenAI-compatible" fetching
            // The plan says: "Parse and wire [custom] as OpenAI-compatible"
            // So if it's custom, we might try to use OpenAIProvider with custom base URL to fetch models?
            // OpenAIProvider doesn't expose list_models as network call yet.
            // So we skip for now.
            warn!(
                provider = %provider_key,
                "dynamic fetch requested but not fully implemented for this provider; falling back to defaults"
            );
            Ok(Vec::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::app_config::{DerivedInterfaceConfig, DerivedModelsConfig};

    fn default_config() -> DerivedAppConfig {
        DerivedAppConfig {
            app_title: None,
            interface: DerivedInterfaceConfig::default(),
            endpoints: vec![],
            model_specs: vec![],
            notices: vec![],
        }
    }

    #[tokio::test]
    async fn test_build_empty_config_allowed() {
        let config = Arc::new(default_config());
        let catalog = ModelCatalog::build(config, true).await;
        assert!(catalog.is_ok());
        assert!(catalog.unwrap().all().is_empty());
    }

    #[tokio::test]
    async fn test_build_empty_config_disallowed() {
        let config = Arc::new(default_config());
        let catalog = ModelCatalog::build(config, false).await;
        assert!(matches!(catalog.unwrap_err(), ModelCatalogError::NoProvidersConfigured));
    }

    #[tokio::test]
    async fn test_build_openai_fetch() {
        let mut config = default_config();
        config.endpoints.push(DerivedEndpoint {
            provider: "openai".to_string(),
            label: "OpenAI".to_string(),
            base_url: None,
            api_key: Some("sk-test".to_string()),
            model_display_label: None,
            icon_url: None,
            models: DerivedModelsConfig {
                fetch: true,
                default: vec![],
                not_allowed: vec![],
                all: false,
            },
            headers: None,
            add_params: None,
            drop_params: vec![],
            original_key: "openai".to_string(),
        });

        let catalog = ModelCatalog::build(Arc::new(config), false).await.expect("failed to build catalog");
        
        let openai_models = catalog.for_provider("openai").expect("missing openai models");
        assert!(!openai_models.is_empty());
        
        // Check for common models
        assert!(openai_models.iter().any(|m| m.id == "gpt-4"));
        assert!(openai_models.iter().any(|m| m.id == "gpt-3.5-turbo"));
    }

    #[tokio::test]
    async fn test_build_openai_defaults() {
        let mut config = default_config();
        config.endpoints.push(DerivedEndpoint {
            provider: "openai".to_string(),
            label: "OpenAI".to_string(),
            base_url: None,
            api_key: Some("sk-test".to_string()),
            model_display_label: None,
            icon_url: None,
            models: DerivedModelsConfig {
                fetch: false,
                default: vec!["gpt-4-custom".to_string()],
                not_allowed: vec![],
                all: false,
            },
            headers: None,
            add_params: None,
            drop_params: vec![],
            original_key: "openai".to_string(),
        });

        let catalog = ModelCatalog::build(Arc::new(config), false).await.expect("failed to build catalog");
        
        let openai_models = catalog.for_provider("openai").expect("missing openai models");
        assert_eq!(openai_models.len(), 1);
        assert_eq!(openai_models[0].id, "gpt-4-custom");
    }

    #[tokio::test]
    async fn test_find_model() {
        let mut config = default_config();
        config.endpoints.push(DerivedEndpoint {
            provider: "openai".to_string(),
            label: "OpenAI".to_string(),
            base_url: None,
            api_key: Some("sk-test".to_string()),
            model_display_label: None,
            icon_url: None,
            models: DerivedModelsConfig {
                fetch: true,
                default: vec![],
                not_allowed: vec![],
                all: false,
            },
            headers: None,
            add_params: None,
            drop_params: vec![],
            original_key: "openai".to_string(),
        });

        let catalog = ModelCatalog::build(Arc::new(config), false).await.expect("failed to build catalog");
        
        let model = catalog.find("openai", "gpt-4");
        assert!(model.is_some());
        assert_eq!(model.unwrap().id, "gpt-4");
        
        let missing = catalog.find("openai", "non-existent");
        assert!(missing.is_none());
    }
}
