use crate::ai::types::ModelInfo;
use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;

pub struct OpenRouterClient {
    api_key: String,
    base_url: String,
    client: Client,
}

impl OpenRouterClient {
    pub fn new(api_key: String, base_url: Option<String>) -> Self {
        let base_url = base_url.unwrap_or_else(|| "https://openrouter.ai/api/v1".to_string());
        Self {
            api_key,
            base_url,
            client: Client::new(),
        }
    }

    pub async fn list_models(&self) -> Result<Vec<ModelInfo>, OpenRouterError> {
        let url = format!("{}/models", self.base_url.trim_end_matches('/'));
        let response = self
            .client
            .get(url)
            .bearer_auth(&self.api_key)
            .send()
            .await
            .map_err(OpenRouterError::RequestFailed)?;

        if !response.status().is_success() {
            return Err(OpenRouterError::HttpStatus(response.status().as_u16()));
        }

        let parsed: ModelsResponse = response.json().await.map_err(OpenRouterError::Decode)?;

        let models = parsed
            .data
            .into_iter()
            .map(|model| {
                let supports_images = model
                    .architecture
                    .as_ref()
                    .map(|arch| {
                        arch.modality
                            .iter()
                            .any(|m| m == "image" || m == "multimodal")
                    })
                    .unwrap_or(false);
                let supports_vision = model
                    .capabilities
                    .as_ref()
                    .and_then(|caps| caps.vision)
                    .unwrap_or(supports_images);
                let supports_functions = model
                    .capabilities
                    .as_ref()
                    .and_then(|caps| caps.function_calling)
                    .unwrap_or(false);

                ModelInfo {
                    id: model.id,
                    display_name: model.name.unwrap_or_else(|| "OpenRouter Model".to_string()),
                    context_window: model.context_length.unwrap_or(0),
                    max_output_tokens: None,
                    supports_streaming: true,
                    supports_images,
                    supports_functions,
                    supports_vision,
                    cost_per_input_token: model.pricing.as_ref().and_then(|p| p.prompt_cost()),
                    cost_per_output_token: model.pricing.as_ref().and_then(|p| p.completion_cost()),
                }
            })
            .collect();

        Ok(models)
    }
}

#[derive(Debug, Error)]
pub enum OpenRouterError {
    #[error("openrouter request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("openrouter responded with unexpected status {0}")]
    HttpStatus(u16),
    #[error("failed to decode openrouter response: {0}")]
    Decode(#[source] reqwest::Error),
}

#[derive(Debug, Deserialize)]
struct ModelsResponse {
    data: Vec<OpenRouterModel>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterModel {
    id: String,
    name: Option<String>,
    description: Option<String>,
    #[serde(default)]
    context_length: Option<u32>,
    pricing: Option<OpenRouterPricing>,
    architecture: Option<OpenRouterArchitecture>,
    capabilities: Option<OpenRouterCapabilities>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterPricing {
    prompt: Option<String>,
    completion: Option<String>,
}

impl OpenRouterPricing {
    fn prompt_cost(&self) -> Option<f64> {
        self.prompt
            .as_ref()
            .and_then(|value| value.parse::<f64>().ok())
    }

    fn completion_cost(&self) -> Option<f64> {
        self.completion
            .as_ref()
            .and_then(|value| value.parse::<f64>().ok())
    }
}

#[derive(Debug, Deserialize)]
struct OpenRouterArchitecture {
    #[serde(default)]
    modality: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterCapabilities {
    #[serde(default)]
    vision: Option<bool>,
    #[serde(default)]
    function_calling: Option<bool>,
}
