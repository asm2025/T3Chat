use crate::ai::providers::AIProvider;
use crate::ai::types::{
    ChatMessage, ChatRequest, ChatResponse, ChatResponseChunk, ModelInfo, ModelParameters,
    TokenUsage,
};
use crate::db::models::MessageRole;
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use anyhow::Context;

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

#[derive(Serialize, Deserialize, Clone)]
struct OpenAIMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

fn build_openai_messages(
    system_message: Option<String>,
    messages: Vec<ChatMessage>,
) -> Vec<OpenAIMessage> {
    let mut system_parts = Vec::new();

    if let Some(sys) = system_message {
        system_parts.push(sys);
    }

    let mut converted: Vec<OpenAIMessage> = Vec::new();

    for msg in messages {
        match msg.role {
            MessageRole::System => system_parts.push(msg.content),
            MessageRole::User => converted.push(OpenAIMessage {
                role: "user".to_string(),
                content: msg.content,
                name: msg.name,
            }),
            MessageRole::Assistant => converted.push(OpenAIMessage {
                role: "assistant".to_string(),
                content: msg.content,
                name: msg.name,
            }),
            MessageRole::Tool => converted.push(OpenAIMessage {
                role: "tool".to_string(),
                content: msg.content,
                name: msg.name,
            }),
        }
    }

    if !system_parts.is_empty() {
        converted.insert(
            0,
            OpenAIMessage {
                role: "system".to_string(),
                content: system_parts.join("\n\n"),
                name: None,
            },
        );
    }

    converted
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

        let ChatRequest {
            model,
            messages,
            parameters,
            system_message,
            ..
        } = request;

        let messages = build_openai_messages(system_message, messages);

        let ModelParameters {
            temperature,
            max_tokens,
            top_p,
            presence_penalty,
            frequency_penalty,
            stop_sequences,
            ..
        } = parameters;

        // O1 (Reasoning) models do not support temperature, top_p, presence_penalty, frequency_penalty
        let is_reasoning_model = model.starts_with("o1");
        
        let (temperature, top_p, presence_penalty, frequency_penalty) = if is_reasoning_model {
            (None, None, None, None)
        } else {
            (temperature, top_p, presence_penalty, frequency_penalty)
        };

        let req = OpenAIRequest {
            model,
            messages,
            temperature,
            max_tokens,
            top_p,
            presence_penalty,
            frequency_penalty,
            stop: stop_sequences,
        };

        let url = format!("{}/chat/completions", self.base_url);
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&req)
            .send()
            .await
            .with_context(|| format!("OpenAI request failed: POST {}", url))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!(
                status = %status,
                body = %truncate_for_log(&error_text, 4_096),
                "OpenAI API error (chat)"
            );
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

        let ChatRequest {
            model,
            messages,
            parameters,
            system_message,
            ..
        } = request;

        let messages = build_openai_messages(system_message, messages);

        let ModelParameters {
            temperature,
            max_tokens,
            top_p,
            presence_penalty,
            frequency_penalty,
            stop_sequences,
            ..
        } = parameters;

        // O1 (Reasoning) models do not support temperature, top_p, presence_penalty, frequency_penalty
        let is_reasoning_model = model.starts_with("o1");
        
        let (temperature, top_p, presence_penalty, frequency_penalty) = if is_reasoning_model {
            (None, None, None, None)
        } else {
            (temperature, top_p, presence_penalty, frequency_penalty)
        };

        let req = OpenAIRequest {
            model: model.clone(),
            messages,
            stream: true,
            temperature,
            max_tokens,
            top_p,
            presence_penalty,
            frequency_penalty,
            stop: stop_sequences,
        };

        let url = format!("{}/chat/completions", self.base_url);
        let model = model.clone();

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&req)
            .send()
            .await
            .with_context(|| format!("OpenAI request failed: POST {}", url))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!(
                status = %status,
                body = %truncate_for_log(&error_text, 4_096),
                "OpenAI API error (stream_chat)"
            );
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

fn truncate_for_log(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    format!("{}...[truncated]", &s[..max])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::models::MessageRole;

    #[test]
    fn test_build_openai_messages() {
        let system = Some("System prompt".to_string());
        let messages = vec![
            ChatMessage {
                role: MessageRole::User,
                content: "Hello".to_string(),
                name: None,
            },
            ChatMessage {
                role: MessageRole::Assistant,
                content: "Hi there".to_string(),
                name: None,
            },
        ];

        let converted = build_openai_messages(system, messages);

        assert_eq!(converted.len(), 3);
        assert_eq!(converted[0].role, "system");
        assert_eq!(converted[0].content, "System prompt");
        assert_eq!(converted[1].role, "user");
        assert_eq!(converted[1].content, "Hello");
        assert_eq!(converted[2].role, "assistant");
        assert_eq!(converted[2].content, "Hi there");
    }
}
