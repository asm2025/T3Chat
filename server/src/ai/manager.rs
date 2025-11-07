use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Result;
use crate::db::schema::AiProvider;
use crate::ai::providers::{AIProvider, openai::OpenAIProvider, anthropic::AnthropicProvider, google::GoogleProvider};

pub enum ProviderWrapper {
    OpenAI(OpenAIProvider),
    Anthropic(AnthropicProvider),
    Google(GoogleProvider),
}

impl ProviderWrapper {
    pub async fn chat(&self, request: crate::ai::types::ChatRequest) -> anyhow::Result<crate::ai::types::ChatResponse> {
        match self {
            ProviderWrapper::OpenAI(p) => p.chat(request).await,
            ProviderWrapper::Anthropic(p) => p.chat(request).await,
            ProviderWrapper::Google(p) => p.chat(request).await,
        }
    }

    pub async fn stream_chat(
        &self,
        request: crate::ai::types::ChatRequest,
    ) -> anyhow::Result<Box<dyn futures::Stream<Item = anyhow::Result<crate::ai::types::ChatResponseChunk>> + Send + Unpin + '_>> {
        match self {
            ProviderWrapper::OpenAI(p) => {
                let stream = p.stream_chat(request).await?;
                Ok(Box::new(stream))
            },
            ProviderWrapper::Anthropic(p) => {
                let stream = p.stream_chat(request).await?;
                Ok(Box::new(stream))
            },
            ProviderWrapper::Google(p) => {
                let stream = p.stream_chat(request).await?;
                Ok(Box::new(stream))
            },
        }
    }

    pub fn get_model_info(&self, model_id: &str) -> Option<crate::ai::types::ModelInfo> {
        match self {
            ProviderWrapper::OpenAI(p) => p.get_model_info(model_id),
            ProviderWrapper::Anthropic(p) => p.get_model_info(model_id),
            ProviderWrapper::Google(p) => p.get_model_info(model_id),
        }
    }

    pub fn list_models(&self) -> Vec<crate::ai::types::ModelInfo> {
        match self {
            ProviderWrapper::OpenAI(p) => p.list_models(),
            ProviderWrapper::Anthropic(p) => p.list_models(),
            ProviderWrapper::Google(p) => p.list_models(),
        }
    }
}

pub struct ProviderManager {
    providers: HashMap<AiProvider, Arc<ProviderWrapper>>,
}

impl ProviderManager {
    pub fn new(api_keys: HashMap<AiProvider, String>) -> Result<Self> {
        let mut providers = HashMap::new();

        for (provider, api_key) in api_keys {
            let provider_impl = match provider {
                AiProvider::OpenAI => Arc::new(ProviderWrapper::OpenAI(OpenAIProvider::new(api_key))),
                AiProvider::Anthropic => Arc::new(ProviderWrapper::Anthropic(AnthropicProvider::new(api_key))),
                AiProvider::Google => Arc::new(ProviderWrapper::Google(GoogleProvider::new(api_key))),
                _ => continue, // Skip unsupported providers for now
            };
            providers.insert(provider, provider_impl);
        }

        Ok(Self { providers })
    }

    pub fn get_provider(&self, provider: &AiProvider) -> Option<Arc<ProviderWrapper>> {
        self.providers.get(provider).cloned()
    }
}

