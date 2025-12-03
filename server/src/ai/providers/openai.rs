use crate::ai::providers::AIProvider;
use crate::ai::types::{ChatRequest, ChatResponse, ChatResponseChunk, ModelInfo, TokenUsage};
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::pin::Pin;

pub struct OpenAIProvider {
    api_key: String,
    client: reqwest::Client,
    base_url: String,
}

impl OpenAIProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
            base_url: "https://api.openai.com/v1".to_string(),
        }
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }
}

#[async_trait]
impl AIProvider for OpenAIProvider {
    fn name(&self) -> &str {
        "openai"
    }

    async fn chat(&self, request: ChatRequest) -> anyhow::Result<ChatResponse> {
        #[derive(Serialize)]
        struct OpenAIRequest {
            model: String,
            messages: Vec<OpenAIMessage>,
            #[serde(skip_serializing_if = "Option::is_none")]
            temperature: Option<f32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            max_tokens: Option<u32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            top_p: Option<f32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            presence_penalty: Option<f32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            frequency_penalty: Option<f32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            stop: Option<Vec<String>>,
        }

        #[derive(Serialize, Deserialize)]
        struct OpenAIMessage {
            role: String,
            content: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            name: Option<String>,
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
            prompt_tokens: u32,
            completion_tokens: u32,
            total_tokens: u32,
        }

        // Convert messages, prepending system message if provided
        let mut messages: Vec<OpenAIMessage> = Vec::new();

        if let Some(system_msg) = request.system_message {
            messages.push(OpenAIMessage {
                role: "system".to_string(),
                content: system_msg,
                name: None,
            });
        }

        messages.extend(request.messages.into_iter().map(|m| OpenAIMessage {
            role: match m.role {
                crate::db::models::MessageRole::User => "user".to_string(),
                crate::db::models::MessageRole::Assistant => "assistant".to_string(),
                crate::db::models::MessageRole::System => "system".to_string(),
                crate::db::models::MessageRole::Tool => "tool".to_string(),
            },
            content: m.content,
            name: m.name,
        }));

        let req = OpenAIRequest {
            model: request.model,
            messages,
            temperature: request.parameters.temperature,
            max_tokens: request.parameters.max_tokens,
            top_p: request.parameters.top_p,
            presence_penalty: request.parameters.presence_penalty,
            frequency_penalty: request.parameters.frequency_penalty,
            stop: request.parameters.stop_sequences,
        };

        let url = format!("{}/chat/completions", self.base_url);
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&req)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("OpenAI API error ({}): {}", status, error_text);
        }

        let response: OpenAIResponse = response.json().await?;

        Ok(ChatResponse {
            content: response
                .choices
                .first()
                .map(|c| c.message.content.clone())
                .unwrap_or_default(),
            model: response.model,
            usage: response.usage.map(|u| TokenUsage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
            }),
            finish_reason: response
                .choices
                .first()
                .and_then(|c| c.finish_reason.clone()),
        })
    }

    async fn stream_chat(
        &self,
        request: ChatRequest,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<ChatResponseChunk>> + Send>>> {
        #[derive(Serialize)]
        struct OpenAIRequest {
            model: String,
            messages: Vec<OpenAIMessage>,
            stream: bool,
            #[serde(skip_serializing_if = "Option::is_none")]
            temperature: Option<f32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            max_tokens: Option<u32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            top_p: Option<f32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            presence_penalty: Option<f32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            frequency_penalty: Option<f32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            stop: Option<Vec<String>>,
        }

        #[derive(Serialize)]
        struct OpenAIMessage {
            role: String,
            content: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            name: Option<String>,
        }

        #[derive(Deserialize)]
        struct StreamResponse {
            choices: Vec<StreamChoice>,
            #[serde(default)]
            model: String,
        }

        #[derive(Deserialize)]
        struct StreamChoice {
            delta: StreamDelta,
            finish_reason: Option<String>,
        }

        #[derive(Deserialize)]
        struct StreamDelta {
            #[serde(default)]
            content: String,
        }

        // Convert messages
        let mut messages: Vec<OpenAIMessage> = Vec::new();

        if let Some(system_msg) = request.system_message {
            messages.push(OpenAIMessage {
                role: "system".to_string(),
                content: system_msg,
                name: None,
            });
        }

        messages.extend(request.messages.into_iter().map(|m| OpenAIMessage {
            role: match m.role {
                crate::db::models::MessageRole::User => "user".to_string(),
                crate::db::models::MessageRole::Assistant => "assistant".to_string(),
                crate::db::models::MessageRole::System => "system".to_string(),
                crate::db::models::MessageRole::Tool => "tool".to_string(),
            },
            content: m.content,
            name: m.name,
        }));

        let req = OpenAIRequest {
            model: request.model.clone(),
            messages,
            stream: true,
            temperature: request.parameters.temperature,
            max_tokens: request.parameters.max_tokens,
            top_p: request.parameters.top_p,
            presence_penalty: request.parameters.presence_penalty,
            frequency_penalty: request.parameters.frequency_penalty,
            stop: request.parameters.stop_sequences,
        };

        let url = format!("{}/chat/completions", self.base_url);
        let model = request.model.clone();

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&req)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("OpenAI API error ({}): {}", status, error_text);
        }

        let stream = response
            .bytes_stream()
            .map(move |result| {
                match result {
                    Ok(bytes) => {
                        let text = String::from_utf8_lossy(bytes.as_ref());

                        // Parse SSE format
                        let mut chunks = Vec::new();
                        for line in text.lines() {
                            if line.starts_with("data: ") {
                                let data = &line[6..];
                                if data == "[DONE]" {
                                    chunks.push(Ok(ChatResponseChunk {
                                        delta: String::new(),
                                        done: true,
                                        model: Some(model.clone()),
                                        finish_reason: Some("stop".to_string()),
                                    }));
                                    break;
                                }

                                if let Ok(parsed) = serde_json::from_str::<StreamResponse>(data) {
                                    if let Some(choice) = parsed.choices.first() {
                                        chunks.push(Ok(ChatResponseChunk {
                                            delta: choice.delta.content.clone(),
                                            done: choice.finish_reason.is_some(),
                                            model: Some(parsed.model.clone()),
                                            finish_reason: choice.finish_reason.clone(),
                                        }));
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
        // Common OpenAI models (updated list)
        match model_id {
            "gpt-4-turbo" | "gpt-4-turbo-preview" => Some(ModelInfo {
                id: model_id.to_string(),
                display_name: "GPT-4 Turbo".to_string(),
                context_window: 128000,
                max_output_tokens: Some(4096),
                supports_streaming: true,
                supports_images: true,
                supports_functions: true,
                supports_vision: true,
                cost_per_input_token: Some(0.00001),
                cost_per_output_token: Some(0.00003),
            }),
            "gpt-4" => Some(ModelInfo {
                id: "gpt-4".to_string(),
                display_name: "GPT-4".to_string(),
                context_window: 8192,
                max_output_tokens: Some(8192),
                supports_streaming: true,
                supports_images: false,
                supports_functions: true,
                supports_vision: false,
                cost_per_input_token: Some(0.00003),
                cost_per_output_token: Some(0.00006),
            }),
            "gpt-4o" => Some(ModelInfo {
                id: "gpt-4o".to_string(),
                display_name: "GPT-4o".to_string(),
                context_window: 128000,
                max_output_tokens: Some(16384),
                supports_streaming: true,
                supports_images: true,
                supports_functions: true,
                supports_vision: true,
                cost_per_input_token: Some(0.000005),
                cost_per_output_token: Some(0.000015),
            }),
            "gpt-3.5-turbo" => Some(ModelInfo {
                id: "gpt-3.5-turbo".to_string(),
                display_name: "GPT-3.5 Turbo".to_string(),
                context_window: 16385,
                max_output_tokens: Some(4096),
                supports_streaming: true,
                supports_images: false,
                supports_functions: true,
                supports_vision: false,
                cost_per_input_token: Some(0.0000005),
                cost_per_output_token: Some(0.0000015),
            }),
            "o1-preview" | "o1" => Some(ModelInfo {
                id: model_id.to_string(),
                display_name: "o1 (Reasoning)".to_string(),
                context_window: 128000,
                max_output_tokens: Some(32768),
                supports_streaming: true,
                supports_images: false,
                supports_functions: false,
                supports_vision: false,
                cost_per_input_token: Some(0.000015),
                cost_per_output_token: Some(0.00006),
            }),
            _ => None,
        }
    }

    fn list_models(&self) -> Vec<ModelInfo> {
        vec![
            self.get_model_info("gpt-4-turbo").unwrap(),
            self.get_model_info("gpt-4o").unwrap(),
            self.get_model_info("gpt-4").unwrap(),
            self.get_model_info("gpt-3.5-turbo").unwrap(),
            self.get_model_info("o1-preview").unwrap(),
        ]
    }

    async fn validate_api_key(&self, api_key: &str) -> anyhow::Result<bool> {
        let url = format!("{}/models", self.base_url);
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await?;

        Ok(response.status().is_success())
    }
}
