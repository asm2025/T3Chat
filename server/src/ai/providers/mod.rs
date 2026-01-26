use crate::ai::types::{
    AgentEvent, ChatRequest, ChatResponse, ChatResponseChunk, ModelInfo, TokenUsage,
};
use crate::db::models::Tool;
use async_trait::async_trait;
use futures::{Stream, StreamExt};
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
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<ChatResponseChunk>> + Send>>>;

    /// Stream chat with tool execution support
    async fn stream_chat_with_tools(
        &self,
        request: ChatRequest,
        _available_tools: Vec<Tool>,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<AgentEvent>> + Send>>> {
        let mut stream = self.stream_chat(request).await?;

        let event_stream = async_stream::stream! {
            let mut full_content = String::new();
            let mut finish_reason: Option<String> = None;
            let usage: Option<TokenUsage> = None;
            let mut sent_complete = false;

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        if !chunk.delta.is_empty() {
                            full_content.push_str(&chunk.delta);
                            yield Ok(AgentEvent::TextDelta { delta: chunk.delta });
                        }

                        if chunk.finish_reason.is_some() {
                            finish_reason = chunk.finish_reason.clone();
                        }

                        if chunk.done && !sent_complete {
                            sent_complete = true;
                            yield Ok(AgentEvent::MessageComplete {
                                content: full_content.clone(),
                                finish_reason: finish_reason.clone(),
                                usage: usage.clone(),
                            });
                        }
                    }
                    Err(e) => {
                        yield Err(e);
                        break;
                    }
                }
            }

            if !sent_complete && !full_content.is_empty() {
                yield Ok(AgentEvent::MessageComplete {
                    content: full_content,
                    finish_reason,
                    usage,
                });
            }
        };

        Ok(Box::pin(event_stream))
    }

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
pub mod chatllm;
pub mod google;
pub mod openai;
pub mod openrouter;
