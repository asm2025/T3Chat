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

/// RouteLLM (Abacus.ai) provider implementation.
///
/// This client targets the OpenAI-compatible
/// `https://routellm.abacus.ai/v1/chat/completions` endpoint.
/// The base URL is configurable so self-hosted proxies can be supported.
pub struct RouteLLMProvider {
    api_key: String,
    client: reqwest::Client,
    base_url: String,
}

impl RouteLLMProvider {
    pub fn new(api_key: String, base_url: Option<String>) -> Self {
        let base_url = base_url.unwrap_or_else(|| "https://routellm.abacus.ai/v1".to_string());
        Self {
            api_key,
            client: reqwest::Client::new(),
            base_url,
        }
    }

    pub async fn fetch_models(&self) -> anyhow::Result<Vec<ModelInfo>> {
        #[derive(Deserialize)]
        struct ModelsResponse {
            data: Vec<ModelData>,
        }

        #[derive(Deserialize)]
        struct ModelData {
            id: String,
        }

        let url = format!("{}/models", self.base_url.trim_end_matches('/'));
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Failed to fetch models: {} - {}", status, text);
        }

        let parsed: ModelsResponse = response.json().await?;

        let models = parsed
            .data
            .into_iter()
            .map(|m| ModelInfo {
                id: m.id.clone(),
                display_name: m.id,
                context_window: 128000, // Default assumption for RouteLLM/modern models
                max_output_tokens: None,
                supports_streaming: true,
                supports_images: false, // We can't know for sure without more metadata
                supports_functions: false,
                supports_vision: false,
                cost_per_input_token: None,
                cost_per_output_token: None,
            })
            .collect();

        Ok(models)
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct RouteLLMMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

fn build_routellm_messages(
    system_message: Option<String>,
    messages: Vec<ChatMessage>,
) -> Vec<RouteLLMMessage> {
    let mut system_parts = Vec::new();

    if let Some(sys) = system_message {
        system_parts.push(sys);
    }

    let mut converted: Vec<RouteLLMMessage> = Vec::new();

    for msg in messages {
        match msg.role {
            MessageRole::System => system_parts.push(msg.content),
            MessageRole::User => converted.push(RouteLLMMessage {
                role: "user".to_string(),
                content: msg.content,
                name: msg.name,
            }),
            MessageRole::Assistant => converted.push(RouteLLMMessage {
                role: "assistant".to_string(),
                content: msg.content,
                name: msg.name,
            }),
            MessageRole::Tool => converted.push(RouteLLMMessage {
                role: "tool".to_string(),
                content: msg.content,
                name: msg.name,
            }),
        }
    }

    if !system_parts.is_empty() {
        converted.insert(
            0,
            RouteLLMMessage {
                role: "system".to_string(),
                content: system_parts.join("\n\n"),
                name: None,
            },
        );
    }

    converted
}

#[async_trait]
impl AIProvider for RouteLLMProvider {
    fn name(&self) -> &str {
        "routellm"
    }

    async fn chat(&self, request: ChatRequest) -> anyhow::Result<ChatResponse> {
        #[derive(Serialize)]
        struct RouteLLMRequest {
            model: String,
            messages: Vec<RouteLLMMessage>,
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
            #[serde(skip_serializing_if = "Option::is_none")]
            stream: Option<bool>,
        }

        #[derive(Deserialize)]
        struct RouteLLMResponse {
            choices: Vec<RouteLLMChoice>,
            model: String,
            #[serde(default)]
            usage: Option<RouteLLMUsage>,
        }

        #[derive(Deserialize)]
        struct RouteLLMChoice {
            message: RouteLLMMessage,
            #[serde(default)]
            finish_reason: Option<String>,
        }

        #[derive(Deserialize)]
        struct RouteLLMUsage {
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

        let messages = build_routellm_messages(system_message, messages);

        let ModelParameters {
            temperature,
            max_tokens,
            top_p,
            presence_penalty,
            frequency_penalty,
            stop_sequences,
            ..
        } = parameters;

        let req = RouteLLMRequest {
            model,
            messages,
            temperature,
            max_tokens,
            top_p,
            presence_penalty,
            frequency_penalty,
            stop: stop_sequences,
            stream: Some(false),
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
            anyhow::bail!("RouteLLM API error ({}): {}", status, error_text);
        }

        let response: RouteLLMResponse = response.json().await?;

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
        struct RouteLLMRequest {
            model: String,
            messages: Vec<RouteLLMMessage>,
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
            #[serde(default)]
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

        let messages = build_routellm_messages(system_message, messages);

        let ModelParameters {
            temperature,
            max_tokens,
            top_p,
            presence_penalty,
            frequency_penalty,
            stop_sequences,
            ..
        } = parameters;

        let req = RouteLLMRequest {
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
        let model_clone = model.clone();

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
            anyhow::bail!("RouteLLM API error ({}): {}", status, error_text);
        }

        let stream = response
            .bytes_stream()
            .map(move |result| match result {
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(bytes.as_ref());

                    let mut chunks = Vec::new();
                    for line in text.lines() {
                        if line.starts_with("data: ") {
                            let data = &line[6..];
                            if data == "[DONE]" {
                                chunks.push(Ok(ChatResponseChunk {
                                    delta: String::new(),
                                    done: true,
                                    model: Some(model_clone.clone()),
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
                Err(e) => futures::stream::iter(vec![Err(anyhow::anyhow!("Stream error: {}", e))]),
            })
            .flatten();

        Ok(Box::pin(stream))
    }

    fn get_model_info(&self, _model_id: &str) -> Option<ModelInfo> {
        // RouteLLM models/routes are described via t3chat.yaml and the model catalog.
        // We rely on that metadata instead of duplicating it here.
        None
    }

    fn list_models(&self) -> Vec<ModelInfo> {
        // RouteLLM models/routes are provided by the model catalog; this client
        // does not expose its own static list.
        Vec::new()
    }

    async fn validate_api_key(&self, _api_key: &str) -> anyhow::Result<bool> {
        // For now, assume the configured key is valid; RouteLLM errors will be
        // surfaced at request time.
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::models::MessageRole;

    #[test]
    fn test_build_routellm_messages() {
        let system = Some("System prompt".to_string());
        let messages = vec![
            ChatMessage {
                role: MessageRole::User,
                content: "Hello".to_string(),
                name: None,
            },
            ChatMessage {
                role: MessageRole::Assistant,
                content: "Hi".to_string(),
                name: None,
            },
        ];

        let converted = build_routellm_messages(system, messages);

        assert_eq!(converted.len(), 3);
        assert_eq!(converted[0].role, "system");
        assert_eq!(converted[0].content, "System prompt");
        assert_eq!(converted[1].role, "user");
        assert_eq!(converted[1].content, "Hello");
        assert_eq!(converted[2].role, "assistant");
        assert_eq!(converted[2].content, "Hi");
    }

    #[test]
    fn test_new_provider_default_url() {
        let p = RouteLLMProvider::new("sk-test".to_string(), None);
        assert_eq!(p.base_url, "https://routellm.abacus.ai/v1");
    }

    #[test]
    fn test_new_provider_custom_url() {
        let p = RouteLLMProvider::new("sk-test".to_string(), Some("http://localhost:8000".to_string()));
        assert_eq!(p.base_url, "http://localhost:8000");
    }
}
