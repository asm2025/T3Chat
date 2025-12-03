use crate::ai::providers::AIProvider;
use crate::ai::types::{ChatRequest, ChatResponse, ChatResponseChunk, ModelInfo, TokenUsage};
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::pin::Pin;

pub struct GoogleProvider {
    api_key: String,
    client: reqwest::Client,
    base_url: String,
}

impl GoogleProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
            base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
        }
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }
}

#[async_trait]
impl AIProvider for GoogleProvider {
    fn name(&self) -> &str {
        "google"
    }

    async fn chat(&self, request: ChatRequest) -> anyhow::Result<ChatResponse> {
        #[derive(Serialize)]
        struct GoogleRequest {
            contents: Vec<GoogleContent>,
            #[serde(skip_serializing_if = "Option::is_none")]
            system_instruction: Option<SystemInstruction>,
            generation_config: GoogleGenerationConfig,
        }

        #[derive(Serialize)]
        struct SystemInstruction {
            parts: Vec<GooglePart>,
        }

        #[derive(Serialize)]
        struct GoogleContent {
            role: String,
            parts: Vec<GooglePart>,
        }

        #[derive(Serialize, Deserialize)]
        struct GooglePart {
            text: String,
        }

        #[derive(Serialize)]
        struct GoogleGenerationConfig {
            #[serde(skip_serializing_if = "Option::is_none")]
            temperature: Option<f32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            max_output_tokens: Option<u32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            top_p: Option<f32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            top_k: Option<u32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            stop_sequences: Option<Vec<String>>,
        }

        #[derive(Deserialize)]
        struct GoogleResponse {
            candidates: Vec<GoogleCandidate>,
            #[serde(rename = "usageMetadata")]
            usage_metadata: Option<GoogleUsageMetadata>,
        }

        #[derive(Deserialize)]
        struct GoogleCandidate {
            content: GoogleResponseContent,
            #[serde(rename = "finishReason")]
            finish_reason: Option<String>,
        }

        #[derive(Deserialize)]
        struct GoogleResponseContent {
            parts: Vec<GooglePart>,
        }

        #[derive(Deserialize)]
        struct GoogleUsageMetadata {
            #[serde(rename = "promptTokenCount")]
            prompt_token_count: u32,
            #[serde(rename = "candidatesTokenCount")]
            candidates_token_count: u32,
            #[serde(rename = "totalTokenCount")]
            total_token_count: u32,
        }

        // Collect system messages and create system instruction
        let (system_instruction, contents): (Option<SystemInstruction>, Vec<_>) = {
            let mut system_parts = Vec::new();
            let mut user_contents = Vec::new();

            if let Some(sys) = request.system_message {
                system_parts.push(GooglePart { text: sys });
            }

            for msg in request.messages {
                match msg.role {
                    crate::db::models::MessageRole::System => {
                        system_parts.push(GooglePart { text: msg.content });
                    }
                    crate::db::models::MessageRole::User => {
                        user_contents.push(GoogleContent {
                            role: "user".to_string(),
                            parts: vec![GooglePart { text: msg.content }],
                        });
                    }
                    crate::db::models::MessageRole::Assistant => {
                        user_contents.push(GoogleContent {
                            role: "model".to_string(),
                            parts: vec![GooglePart { text: msg.content }],
                        });
                    }
                    crate::db::models::MessageRole::Tool => {
                        // Google doesn't have tool role in the same way
                        user_contents.push(GoogleContent {
                            role: "user".to_string(),
                            parts: vec![GooglePart { text: msg.content }],
                        });
                    }
                }
            }

            let sys_inst = if !system_parts.is_empty() {
                Some(SystemInstruction {
                    parts: system_parts,
                })
            } else {
                None
            };

            (sys_inst, user_contents)
        };

        let req = GoogleRequest {
            contents,
            system_instruction,
            generation_config: GoogleGenerationConfig {
                temperature: request.parameters.temperature,
                max_output_tokens: request.parameters.max_tokens,
                top_p: request.parameters.top_p,
                top_k: request.parameters.top_k,
                stop_sequences: request.parameters.stop_sequences,
            },
        };

        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.base_url, request.model, self.api_key
        );

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&req)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Google API error ({}): {}", status, error_text);
        }

        let response: GoogleResponse = response.json().await?;

        Ok(ChatResponse {
            content: response
                .candidates
                .first()
                .and_then(|c| c.content.parts.first())
                .map(|p| p.text.clone())
                .unwrap_or_default(),
            model: request.model.clone(),
            usage: response.usage_metadata.map(|u| TokenUsage {
                prompt_tokens: u.prompt_token_count,
                completion_tokens: u.candidates_token_count,
                total_tokens: u.total_token_count,
            }),
            finish_reason: response
                .candidates
                .first()
                .and_then(|c| c.finish_reason.clone()),
        })
    }

    async fn stream_chat(
        &self,
        request: ChatRequest,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<ChatResponseChunk>> + Send>>> {
        #[derive(Serialize)]
        struct GoogleRequest {
            contents: Vec<GoogleContent>,
            #[serde(skip_serializing_if = "Option::is_none")]
            system_instruction: Option<SystemInstruction>,
            generation_config: GoogleGenerationConfig,
        }

        #[derive(Serialize)]
        struct SystemInstruction {
            parts: Vec<GooglePart>,
        }

        #[derive(Serialize)]
        struct GoogleContent {
            role: String,
            parts: Vec<GooglePart>,
        }

        #[derive(Serialize)]
        struct GooglePart {
            text: String,
        }

        #[derive(Serialize)]
        struct GoogleGenerationConfig {
            #[serde(skip_serializing_if = "Option::is_none")]
            temperature: Option<f32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            max_output_tokens: Option<u32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            top_p: Option<f32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            top_k: Option<u32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            stop_sequences: Option<Vec<String>>,
        }

        #[derive(Deserialize)]
        struct GoogleStreamResponse {
            candidates: Vec<GoogleCandidate>,
        }

        #[derive(Deserialize)]
        struct GoogleCandidate {
            content: GoogleResponseContent,
            #[serde(rename = "finishReason")]
            finish_reason: Option<String>,
        }

        #[derive(Deserialize)]
        struct GoogleResponseContent {
            parts: Vec<GoogleResponsePart>,
        }

        #[derive(Deserialize)]
        struct GoogleResponsePart {
            text: String,
        }

        // Collect system messages
        let (system_instruction, contents): (Option<SystemInstruction>, Vec<_>) = {
            let mut system_parts = Vec::new();
            let mut user_contents = Vec::new();

            if let Some(sys) = request.system_message {
                system_parts.push(GooglePart { text: sys });
            }

            for msg in request.messages {
                match msg.role {
                    crate::db::models::MessageRole::System => {
                        system_parts.push(GooglePart { text: msg.content });
                    }
                    crate::db::models::MessageRole::User => {
                        user_contents.push(GoogleContent {
                            role: "user".to_string(),
                            parts: vec![GooglePart { text: msg.content }],
                        });
                    }
                    crate::db::models::MessageRole::Assistant => {
                        user_contents.push(GoogleContent {
                            role: "model".to_string(),
                            parts: vec![GooglePart { text: msg.content }],
                        });
                    }
                    crate::db::models::MessageRole::Tool => {
                        user_contents.push(GoogleContent {
                            role: "user".to_string(),
                            parts: vec![GooglePart { text: msg.content }],
                        });
                    }
                }
            }

            let sys_inst = if !system_parts.is_empty() {
                Some(SystemInstruction {
                    parts: system_parts,
                })
            } else {
                None
            };

            (sys_inst, user_contents)
        };

        let req = GoogleRequest {
            contents,
            system_instruction,
            generation_config: GoogleGenerationConfig {
                temperature: request.parameters.temperature,
                max_output_tokens: request.parameters.max_tokens,
                top_p: request.parameters.top_p,
                top_k: request.parameters.top_k,
                stop_sequences: request.parameters.stop_sequences,
            },
        };

        let url = format!(
            "{}/models/{}:streamGenerateContent?key={}&alt=sse",
            self.base_url, request.model, self.api_key
        );

        let model = request.model.clone();

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&req)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Google API error ({}): {}", status, error_text);
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

                                if let Ok(parsed) =
                                    serde_json::from_str::<GoogleStreamResponse>(data)
                                {
                                    if let Some(candidate) = parsed.candidates.first() {
                                        if let Some(part) = candidate.content.parts.first() {
                                            chunks.push(Ok(ChatResponseChunk {
                                                delta: part.text.clone(),
                                                done: candidate.finish_reason.is_some(),
                                                model: Some(model.clone()),
                                                finish_reason: candidate.finish_reason.clone(),
                                            }));
                                        }
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
            "gemini-2.0-flash" | "gemini-2.0-flash-exp" => Some(ModelInfo {
                id: model_id.to_string(),
                display_name: "Gemini 2.0 Flash".to_string(),
                context_window: 1048576, // 1M tokens
                max_output_tokens: Some(8192),
                supports_streaming: true,
                supports_images: true,
                supports_functions: true,
                supports_vision: true,
                cost_per_input_token: Some(0.000000075),
                cost_per_output_token: Some(0.0000003),
            }),
            "gemini-1.5-pro" | "gemini-1.5-pro-latest" => Some(ModelInfo {
                id: model_id.to_string(),
                display_name: "Gemini 1.5 Pro".to_string(),
                context_window: 2097152, // 2M tokens
                max_output_tokens: Some(8192),
                supports_streaming: true,
                supports_images: true,
                supports_functions: true,
                supports_vision: true,
                cost_per_input_token: Some(0.00000125),
                cost_per_output_token: Some(0.000005),
            }),
            "gemini-1.5-flash" | "gemini-1.5-flash-latest" => Some(ModelInfo {
                id: model_id.to_string(),
                display_name: "Gemini 1.5 Flash".to_string(),
                context_window: 1048576,
                max_output_tokens: Some(8192),
                supports_streaming: true,
                supports_images: true,
                supports_functions: true,
                supports_vision: true,
                cost_per_input_token: Some(0.000000075),
                cost_per_output_token: Some(0.0000003),
            }),
            "gemini-pro" => Some(ModelInfo {
                id: "gemini-pro".to_string(),
                display_name: "Gemini Pro".to_string(),
                context_window: 32768,
                max_output_tokens: Some(8192),
                supports_streaming: true,
                supports_images: false,
                supports_functions: true,
                supports_vision: false,
                cost_per_input_token: Some(0.0000005),
                cost_per_output_token: Some(0.0000015),
            }),
            "gemini-pro-vision" => Some(ModelInfo {
                id: "gemini-pro-vision".to_string(),
                display_name: "Gemini Pro Vision".to_string(),
                context_window: 16384,
                max_output_tokens: Some(4096),
                supports_streaming: true,
                supports_images: true,
                supports_functions: false,
                supports_vision: true,
                cost_per_input_token: Some(0.00000025),
                cost_per_output_token: Some(0.0000005),
            }),
            _ => None,
        }
    }

    fn list_models(&self) -> Vec<ModelInfo> {
        vec![
            self.get_model_info("gemini-2.0-flash").unwrap(),
            self.get_model_info("gemini-1.5-pro").unwrap(),
            self.get_model_info("gemini-1.5-flash").unwrap(),
            self.get_model_info("gemini-pro").unwrap(),
            self.get_model_info("gemini-pro-vision").unwrap(),
        ]
    }

    async fn validate_api_key(&self, api_key: &str) -> anyhow::Result<bool> {
        let url = format!("{}/models?key={}", self.base_url, api_key);
        let response = self.client.get(&url).send().await?;

        Ok(response.status().is_success())
    }
}
