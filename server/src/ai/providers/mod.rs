use crate::ai::types::{ChatRequest, ChatResponse, ChatResponseChunk, ModelInfo};
use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;

/// AI Provider trait - all providers must implement this
#[async_trait]
pub trait AIProvider: Send + Sync {
    /// Provider name (e.g., "openai", "anthropic", "google")
    fn name(&self) -> &str;

    /// Non-streaming chat completion
    async fn chat(&self, request: ChatRequest) -> anyhow::Result<ChatResponse>;

    /// Streaming chat completion (Server-Sent Events)
    async fn stream_chat(
        &self,
        request: ChatRequest,
    ) -> anyhow::Result<
        Pin<Box<dyn Stream<Item = anyhow::Result<ChatResponseChunk>> + Send>>,
    >;

    /// Get information about a specific model
    fn get_model_info(&self, model_id: &str) -> Option<ModelInfo>;

    /// List all available models for this provider
    fn list_models(&self) -> Vec<ModelInfo>;

    /// Validate an API key (optional - returns true by default)
    async fn validate_api_key(&self, api_key: &str) -> anyhow::Result<bool> {
        // Default implementation - subclasses can override
        let _ = api_key;
        Ok(true)
    }
}

pub mod anthropic;
pub mod google;
pub mod openai;
