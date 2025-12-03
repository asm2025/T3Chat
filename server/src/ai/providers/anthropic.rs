use crate::ai::providers::AIProvider;
use crate::ai::types::{ChatRequest, ChatResponse, ChatResponseChunk, ModelInfo, TokenUsage};
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::pin::Pin;

pub struct AnthropicProvider {
    api_key: String,
    client: reqwest::Client,
    base_url: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
            base_url: "https://api.anthropic.com/v1".to_string(),
        }
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }
}

#[async_trait]
impl AIProvider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    async fn chat(&self, request: ChatRequest) -> anyhow::Result<ChatResponse> {
        #[derive(Serialize)]
        struct AnthropicRequest {
            model: String,
            messages: Vec<AnthropicMessage>,
            max_tokens: u32,
            #[serde(skip_serializing_if = "Option::is_none")]
            system: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            temperature: Option<f32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            top_p: Option<f32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            top_k: Option<u32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            stop_sequences: Option<Vec<String>>,
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

        // Anthropic requires max_tokens, can't be optional
        let max_tokens = request.parameters.max_tokens.unwrap_or(4096);

        // Filter out system messages and use as system parameter
        let (system_msg, messages): (Option<String>, Vec<_>) = {
            let mut system_parts = Vec::new();
            let mut user_messages = Vec::new();

            // Add explicit system message if provided
            if let Some(sys) = request.system_message {
                system_parts.push(sys);
            }

            for msg in request.messages {
                match msg.role {
                    crate::db::models::MessageRole::System => {
                        system_parts.push(msg.content);
                    }
                    crate::db::models::MessageRole::User => {
                        user_messages.push(AnthropicMessage {
                            role: "user".to_string(),
                            content: msg.content,
                        });
                    }
                    crate::db::models::MessageRole::Assistant => {
                        user_messages.push(AnthropicMessage {
                            role: "assistant".to_string(),
                            content: msg.content,
                        });
                    }
                    crate::db::models::MessageRole::Tool => {
                        // Anthropic doesn't support tool role in the same way
                        user_messages.push(AnthropicMessage {
                            role: "user".to_string(),
                            content: msg.content,
                        });
                    }
                }
            }

            let system = if !system_parts.is_empty() {
                Some(system_parts.join("\n\n"))
            } else {
                None
            };

            (system, user_messages)
        };

        let req = AnthropicRequest {
            model: request.model,
            messages,
            max_tokens,
            system: system_msg,
            temperature: request.parameters.temperature,
            top_p: request.parameters.top_p,
            top_k: request.parameters.top_k,
            stop_sequences: request.parameters.stop_sequences,
        };

        let url = format!("{}/messages", self.base_url);
        let response = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&req)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Anthropic API error ({}): {}", status, error_text);
        }

        let response: AnthropicResponse = response.json().await?;

        Ok(ChatResponse {
            content: response
                .content
                .first()
                .map(|c| c.text.clone())
                .unwrap_or_default(),
            model: response.model,
            usage: Some(TokenUsage {
                prompt_tokens: response.usage.input_tokens,
                completion_tokens: response.usage.output_tokens,
                total_tokens: response.usage.input_tokens + response.usage.output_tokens,
            }),
            finish_reason: response.stop_reason,
        })
    }

    async fn stream_chat(
        &self,
        request: ChatRequest,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<ChatResponseChunk>> + Send>>> {
        #[derive(Serialize)]
        struct AnthropicRequest {
            model: String,
            messages: Vec<AnthropicMessage>,
            max_tokens: u32,
            stream: bool,
            #[serde(skip_serializing_if = "Option::is_none")]
            system: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            temperature: Option<f32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            top_p: Option<f32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            top_k: Option<u32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            stop_sequences: Option<Vec<String>>,
        }

        #[derive(Serialize)]
        struct AnthropicMessage {
            role: String,
            content: String,
        }

        #[derive(Deserialize)]
        struct StreamEvent {
            #[serde(rename = "type")]
            event_type: String,
            delta: Option<StreamDelta>,
            message: Option<StreamMessage>,
        }

        #[derive(Deserialize)]
        struct StreamDelta {
            #[serde(default)]
            text: String,
            stop_reason: Option<String>,
        }

        #[derive(Deserialize)]
        struct StreamMessage {
            model: String,
        }

        let max_tokens = request.parameters.max_tokens.unwrap_or(4096);

        // Filter out system messages
        let (system_msg, messages): (Option<String>, Vec<_>) = {
            let mut system_parts = Vec::new();
            let mut user_messages = Vec::new();

            if let Some(sys) = request.system_message {
                system_parts.push(sys);
            }

            for msg in request.messages {
                match msg.role {
                    crate::db::models::MessageRole::System => {
                        system_parts.push(msg.content);
                    }
                    crate::db::models::MessageRole::User => {
                        user_messages.push(AnthropicMessage {
                            role: "user".to_string(),
                            content: msg.content,
                        });
                    }
                    crate::db::models::MessageRole::Assistant => {
                        user_messages.push(AnthropicMessage {
                            role: "assistant".to_string(),
                            content: msg.content,
                        });
                    }
                    crate::db::models::MessageRole::Tool => {
                        user_messages.push(AnthropicMessage {
                            role: "user".to_string(),
                            content: msg.content,
                        });
                    }
                }
            }

            let system = if !system_parts.is_empty() {
                Some(system_parts.join("\n\n"))
            } else {
                None
            };

            (system, user_messages)
        };

        let req = AnthropicRequest {
            model: request.model.clone(),
            messages,
            max_tokens,
            stream: true,
            system: system_msg,
            temperature: request.parameters.temperature,
            top_p: request.parameters.top_p,
            top_k: request.parameters.top_k,
            stop_sequences: request.parameters.stop_sequences,
        };

        let url = format!("{}/messages", self.base_url);
        let model = request.model.clone();

        let response = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&req)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Anthropic API error ({}): {}", status, error_text);
        }

        let stream = response
            .bytes_stream()
            .map(move |result| {
                match result {
                    Ok(bytes) => {
                        let text = String::from_utf8_lossy(bytes.as_ref());
                        let mut chunks = Vec::new();

                        // Parse SSE format
                        for line in text.lines() {
                            if line.starts_with("data: ") {
                                let data = &line[6..];

                                if let Ok(event) = serde_json::from_str::<StreamEvent>(data) {
                                    match event.event_type.as_str() {
                                        "content_block_delta" => {
                                            if let Some(delta) = event.delta {
                                                chunks.push(Ok(ChatResponseChunk {
                                                    delta: delta.text,
                                                    done: delta.stop_reason.is_some(),
                                                    model: Some(model.clone()),
                                                    finish_reason: delta.stop_reason,
                                                }));
                                            }
                                        }
                                        "message_stop" => {
                                            chunks.push(Ok(ChatResponseChunk {
                                                delta: String::new(),
                                                done: true,
                                                model: Some(model.clone()),
                                                finish_reason: Some("end_turn".to_string()),
                                            }));
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }

                        futures::stream::iter(chunks)
                    }
                    Err(e) => {
                        futures::stream::iter(vec![Err(anyhow::anyhow!("Stream error: {}", e))])
                    }
                }
            })
            .flatten();

        Ok(Box::pin(stream))
    }

    fn get_model_info(&self, model_id: &str) -> Option<ModelInfo> {
        match model_id {
            "claude-3-5-sonnet-20241022" | "claude-3-5-sonnet" => Some(ModelInfo {
                id: model_id.to_string(),
                display_name: "Claude 3.5 Sonnet".to_string(),
                context_window: 200000,
                max_output_tokens: Some(8192),
                supports_streaming: true,
                supports_images: true,
                supports_functions: true,
                supports_vision: true,
                cost_per_input_token: Some(0.000003),
                cost_per_output_token: Some(0.000015),
            }),
            "claude-3-opus-20240229" | "claude-3-opus" => Some(ModelInfo {
                id: model_id.to_string(),
                display_name: "Claude 3 Opus".to_string(),
                context_window: 200000,
                max_output_tokens: Some(4096),
                supports_streaming: true,
                supports_images: true,
                supports_functions: true,
                supports_vision: true,
                cost_per_input_token: Some(0.000015),
                cost_per_output_token: Some(0.000075),
            }),
            "claude-3-sonnet-20240229" | "claude-3-sonnet" => Some(ModelInfo {
                id: model_id.to_string(),
                display_name: "Claude 3 Sonnet".to_string(),
                context_window: 200000,
                max_output_tokens: Some(4096),
                supports_streaming: true,
                supports_images: true,
                supports_functions: true,
                supports_vision: true,
                cost_per_input_token: Some(0.000003),
                cost_per_output_token: Some(0.000015),
            }),
            "claude-3-haiku-20240307" | "claude-3-haiku" => Some(ModelInfo {
                id: model_id.to_string(),
                display_name: "Claude 3 Haiku".to_string(),
                context_window: 200000,
                max_output_tokens: Some(4096),
                supports_streaming: true,
                supports_images: true,
                supports_functions: true,
                supports_vision: true,
                cost_per_input_token: Some(0.00000025),
                cost_per_output_token: Some(0.00000125),
            }),
            _ => None,
        }
    }

    fn list_models(&self) -> Vec<ModelInfo> {
        vec![
            self.get_model_info("claude-3-5-sonnet").unwrap(),
            self.get_model_info("claude-3-opus").unwrap(),
            self.get_model_info("claude-3-sonnet").unwrap(),
            self.get_model_info("claude-3-haiku").unwrap(),
        ]
    }

    async fn validate_api_key(&self, api_key: &str) -> anyhow::Result<bool> {
        // Anthropic doesn't have a simple validation endpoint
        // We can try a minimal request to check the key
        let url = format!("{}/messages", self.base_url);

        #[derive(Serialize)]
        struct TestRequest {
            model: String,
            messages: Vec<TestMessage>,
            max_tokens: u32,
        }

        #[derive(Serialize)]
        struct TestMessage {
            role: String,
            content: String,
        }

        let test_req = TestRequest {
            model: "claude-3-haiku-20240307".to_string(),
            messages: vec![TestMessage {
                role: "user".to_string(),
                content: "Hi".to_string(),
            }],
            max_tokens: 1,
        };

        let response = self
            .client
            .post(&url)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&test_req)
            .send()
            .await?;

        Ok(response.status().is_success())
    }
}
