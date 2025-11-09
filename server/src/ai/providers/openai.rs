use crate::ai::providers::AIProvider;
use crate::ai::types::{ChatRequest, ChatResponse, ChatResponseChunk, ModelInfo};
use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};

pub struct OpenAIProvider {
    api_key: String,
    client: reqwest::Client,
}

impl OpenAIProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl AIProvider for OpenAIProvider {
    async fn chat(&self, request: ChatRequest) -> anyhow::Result<ChatResponse> {
        #[derive(Serialize)]
        struct OpenAIRequest {
            model: String,
            messages: Vec<OpenAIMessage>,
            temperature: Option<f32>,
            max_tokens: Option<u32>,
        }

        #[derive(Serialize, Deserialize)]
        struct OpenAIMessage {
            role: String,
            content: String,
        }

        #[derive(Deserialize)]
        struct OpenAIResponse {
            choices: Vec<OpenAIChoice>,
            model: String,
            usage: Option<OpenAIUsage>,
        }

        #[derive(Deserialize)]
        struct OpenAIChoice {
            message: OpenAIMessage,
            finish_reason: Option<String>,
        }

        #[derive(Deserialize)]
        struct OpenAIUsage {
            total_tokens: u32,
        }

        let messages: Vec<OpenAIMessage> = request
            .messages
            .into_iter()
            .map(|m| OpenAIMessage {
                role: match m.role {
                    crate::db::models::MessageRole::User => "user".to_string(),
                    crate::db::models::MessageRole::Assistant => "assistant".to_string(),
                    crate::db::models::MessageRole::System => "system".to_string(),
                },
                content: m.content,
            })
            .collect();

        let req = OpenAIRequest {
            model: request.model,
            messages,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&req)
            .send()
            .await?;

        let response: OpenAIResponse = response.json().await?;

        Ok(ChatResponse {
            content: response
                .choices
                .first()
                .map(|c| c.message.content.clone())
                .unwrap_or_default(),
            model: response.model,
            tokens_used: response.usage.map(|u| u.total_tokens),
            finish_reason: response
                .choices
                .first()
                .and_then(|c| c.finish_reason.clone()),
        })
    }

    async fn stream_chat(
        &self,
        request: ChatRequest,
    ) -> anyhow::Result<impl Stream<Item = anyhow::Result<ChatResponseChunk>>> {
        // For now, return a simple implementation
        // Full SSE streaming implementation would go here
        let response = self.chat(request).await?;
        Ok(futures::stream::iter(vec![Ok(ChatResponseChunk {
            content: response.content,
            done: true,
            model: Some(response.model),
        })]))
    }

    fn get_model_info(&self, model_id: &str) -> Option<ModelInfo> {
        // Common OpenAI models
        match model_id {
            "gpt-4" => Some(ModelInfo {
                id: "gpt-4".to_string(),
                display_name: "GPT-4".to_string(),
                context_window: 8192,
                supports_streaming: true,
                supports_images: false,
            }),
            "gpt-3.5-turbo" => Some(ModelInfo {
                id: "gpt-3.5-turbo".to_string(),
                display_name: "GPT-3.5 Turbo".to_string(),
                context_window: 4096,
                supports_streaming: true,
                supports_images: false,
            }),
            _ => None,
        }
    }

    fn list_models(&self) -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: "gpt-4".to_string(),
                display_name: "GPT-4".to_string(),
                context_window: 8192,
                supports_streaming: true,
                supports_images: false,
            },
            ModelInfo {
                id: "gpt-3.5-turbo".to_string(),
                display_name: "GPT-3.5 Turbo".to_string(),
                context_window: 4096,
                supports_streaming: true,
                supports_images: false,
            },
        ]
    }
}
