use crate::ai::providers::{
    anthropic::AnthropicProvider, google::GoogleProvider, openai::OpenAIProvider, AIProvider,
};
use crate::ai::types::{ChatRequest, ChatResponse, ChatResponseChunk, ModelInfo};
use crate::db::models::AiProvider;
use anyhow::Result;
use futures::Stream;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;

/// Wrapper enum for different AI provider implementations
pub enum ProviderWrapper {
    OpenAI(OpenAIProvider),
    Anthropic(AnthropicProvider),
    Google(GoogleProvider),
}

impl ProviderWrapper {
    pub fn name(&self) -> &str {
        match self {
            ProviderWrapper::OpenAI(p) => p.name(),
            ProviderWrapper::Anthropic(p) => p.name(),
            ProviderWrapper::Google(p) => p.name(),
        }
    }

    pub async fn chat(&self, request: ChatRequest) -> anyhow::Result<ChatResponse> {
        match self {
            ProviderWrapper::OpenAI(p) => p.chat(request).await,
            ProviderWrapper::Anthropic(p) => p.chat(request).await,
            ProviderWrapper::Google(p) => p.chat(request).await,
        }
    }

    pub async fn stream_chat(
        &self,
        request: ChatRequest,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<ChatResponseChunk>> + Send>>> {
        match self {
            ProviderWrapper::OpenAI(p) => p.stream_chat(request).await,
            ProviderWrapper::Anthropic(p) => p.stream_chat(request).await,
            ProviderWrapper::Google(p) => p.stream_chat(request).await,
        }
    }

    pub fn get_model_info(&self, model_id: &str) -> Option<ModelInfo> {
        match self {
            ProviderWrapper::OpenAI(p) => p.get_model_info(model_id),
            ProviderWrapper::Anthropic(p) => p.get_model_info(model_id),
            ProviderWrapper::Google(p) => p.get_model_info(model_id),
        }
    }

    pub fn list_models(&self) -> Vec<ModelInfo> {
        match self {
            ProviderWrapper::OpenAI(p) => p.list_models(),
            ProviderWrapper::Anthropic(p) => p.list_models(),
            ProviderWrapper::Google(p) => p.list_models(),
        }
    }

    pub async fn validate_api_key(&self, api_key: &str) -> anyhow::Result<bool> {
        match self {
            ProviderWrapper::OpenAI(p) => p.validate_api_key(api_key).await,
            ProviderWrapper::Anthropic(p) => p.validate_api_key(api_key).await,
            ProviderWrapper::Google(p) => p.validate_api_key(api_key).await,
        }
    }
}

/// Provider manager for creating and managing AI provider instances
pub struct ProviderManager {
    providers: HashMap<AiProvider, Arc<ProviderWrapper>>,
}

impl ProviderManager {
    /// Create a new ProviderManager with the given API keys
    pub fn new(api_keys: HashMap<AiProvider, String>) -> Result<Self> {
        let mut providers = HashMap::new();

        for (provider, api_key) in api_keys {
            let provider_impl = match provider {
                AiProvider::OpenAI => {
                    Arc::new(ProviderWrapper::OpenAI(OpenAIProvider::new(api_key)))
                }
                AiProvider::Anthropic => {
                    Arc::new(ProviderWrapper::Anthropic(AnthropicProvider::new(api_key)))
                }
                AiProvider::Google => {
                    Arc::new(ProviderWrapper::Google(GoogleProvider::new(api_key)))
                }
                _ => continue, // Skip unsupported providers for now
            };
            providers.insert(provider, provider_impl);
        }

        Ok(Self { providers })
    }

    /// Get a provider instance by provider enum
    pub fn get_provider(&self, provider: &AiProvider) -> Option<Arc<ProviderWrapper>> {
        self.providers.get(provider).cloned()
    }

    /// Create a single provider instance without storing it
    pub fn create_provider(provider: &AiProvider, api_key: String) -> Result<ProviderWrapper> {
        let provider_impl = match provider {
            AiProvider::OpenAI => ProviderWrapper::OpenAI(OpenAIProvider::new(api_key)),
            AiProvider::Anthropic => ProviderWrapper::Anthropic(AnthropicProvider::new(api_key)),
            AiProvider::Google => ProviderWrapper::Google(GoogleProvider::new(api_key)),
            _ => anyhow::bail!("Unsupported provider: {:?}", provider),
        };
        Ok(provider_impl)
    }
}
