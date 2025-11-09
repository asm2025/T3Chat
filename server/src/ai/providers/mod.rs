use crate::ai::types::{ChatRequest, ChatResponse, ChatResponseChunk, ModelInfo};
use async_trait::async_trait;
use futures::Stream;

#[async_trait]
pub trait AIProvider: Send + Sync {
    async fn chat(&self, request: ChatRequest) -> anyhow::Result<ChatResponse>;
    async fn stream_chat(
        &self,
        request: ChatRequest,
    ) -> anyhow::Result<impl Stream<Item = anyhow::Result<ChatResponseChunk>>>;
    fn get_model_info(&self, model_id: &str) -> Option<ModelInfo>;
    fn list_models(&self) -> Vec<ModelInfo>;
}

pub mod anthropic;
pub mod google;
pub mod openai;
