use crate::ai::providers::AIProvider;
use crate::ai::types::{ChatRequest, ChatResponse, ChatResponseChunk, ModelInfo};
use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};

pub struct AnthropicProvider {
    api_key: String,
    client: reqwest::Client,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl AIProvider for AnthropicProvider {
    async fn chat(&self, request: ChatRequest) -> anyhow::Result<ChatResponse> {
        #[derive(Serialize)]
        struct AnthropicRequest {
            model: String,
            messages: Vec<AnthropicMessage>,
            max_tokens: u32,
            temperature: Option<f32>,
        }

        #[derive(Serialize)]
        struct AnthropicMessage {
            role: String,
            content: String,
        }

        #[derive(Deserialize)]
        struct AnthropicResponse {
            content: Vec<AnthropicContent>,
            model: String,
            usage: AnthropicUsage,
            stop_reason: Option<String>,
        }

        #[derive(Deserialize)]
        struct AnthropicContent {
            text: String,
        }

        #[derive(Deserialize)]
        struct AnthropicUsage {
            input_tokens: u32,
            output_tokens: u32,
        }

        let messages: Vec<AnthropicMessage> = request
            .messages
            .into_iter()
            .map(|m| AnthropicMessage {
                role: match m.role {
                    crate::db::models::MessageRole::User => "user".to_string(),
                    crate::db::models::MessageRole::Assistant => "assistant".to_string(),
                    crate::db::models::MessageRole::System => "system".to_string(),
                },
                content: m.content,
            })
            .collect();

        let req = AnthropicRequest {
            model: request.model,
            messages,
            max_tokens: request.max_tokens.unwrap_or(1024),
            temperature: request.temperature,
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&req)
            .send()
            .await?;

        let response: AnthropicResponse = response.json().await?;

        Ok(ChatResponse {
            content: response
                .content
                .first()
                .map(|c| c.text.clone())
                .unwrap_or_default(),
            model: response.model,
            tokens_used: Some(response.usage.input_tokens + response.usage.output_tokens),
            finish_reason: response.stop_reason,
        })
    }

    async fn stream_chat(
        &self,
        request: ChatRequest,
    ) -> anyhow::Result<impl Stream<Item = anyhow::Result<ChatResponseChunk>>> {
        let response = self.chat(request).await?;
        Ok(futures::stream::iter(vec![Ok(ChatResponseChunk {
            content: response.content,
            done: true,
            model: Some(response.model),
        })]))
    }

    fn get_model_info(&self, model_id: &str) -> Option<ModelInfo> {
        match model_id {
            "claude-3-opus" => Some(ModelInfo {
                id: "claude-3-opus".to_string(),
                display_name: "Claude 3 Opus".to_string(),
                context_window: 200000,
                supports_streaming: true,
                supports_images: true,
            }),
            "claude-3-sonnet" => Some(ModelInfo {
                id: "claude-3-sonnet".to_string(),
                display_name: "Claude 3 Sonnet".to_string(),
                context_window: 200000,
                supports_streaming: true,
                supports_images: true,
            }),
            "claude-3-haiku" => Some(ModelInfo {
                id: "claude-3-haiku".to_string(),
                display_name: "Claude 3 Haiku".to_string(),
                context_window: 200000,
                supports_streaming: true,
                supports_images: true,
            }),
            _ => None,
        }
    }

    fn list_models(&self) -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: "claude-3-opus".to_string(),
                display_name: "Claude 3 Opus".to_string(),
                context_window: 200000,
                supports_streaming: true,
                supports_images: true,
            },
            ModelInfo {
                id: "claude-3-sonnet".to_string(),
                display_name: "Claude 3 Sonnet".to_string(),
                context_window: 200000,
                supports_streaming: true,
                supports_images: true,
            },
            ModelInfo {
                id: "claude-3-haiku".to_string(),
                display_name: "Claude 3 Haiku".to_string(),
                context_window: 200000,
                supports_streaming: true,
                supports_images: true,
            },
        ]
    }
}
