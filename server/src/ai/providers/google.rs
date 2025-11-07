use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};
use crate::ai::providers::AIProvider;
use crate::ai::types::{ChatRequest, ChatResponse, ChatResponseChunk, ModelInfo};

pub struct GoogleProvider {
    api_key: String,
    client: reqwest::Client,
}

impl GoogleProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl AIProvider for GoogleProvider {
    async fn chat(&self, request: ChatRequest) -> anyhow::Result<ChatResponse> {
        #[derive(Serialize)]
        struct GoogleRequest {
            contents: Vec<GoogleContent>,
            generation_config: GoogleGenerationConfig,
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
            temperature: Option<f32>,
            max_output_tokens: Option<u32>,
        }

        #[derive(Deserialize)]
        struct GoogleResponse {
            candidates: Vec<GoogleCandidate>,
            usage_metadata: Option<GoogleUsageMetadata>,
        }

        #[derive(Deserialize)]
        struct GoogleCandidate {
            content: GoogleResponseContent,
            finish_reason: Option<String>,
        }

        #[derive(Deserialize)]
        struct GoogleResponseContent {
            parts: Vec<GooglePart>,
        }

        #[derive(Deserialize)]
        struct GoogleUsageMetadata {
            total_token_count: u32,
        }

        let contents: Vec<GoogleContent> = request
            .messages
            .into_iter()
            .map(|m| GoogleContent {
                role: match m.role {
                    crate::db::schema::MessageRole::User => "user".to_string(),
                    crate::db::schema::MessageRole::Assistant => "model".to_string(),
                    crate::db::schema::MessageRole::System => "user".to_string(),
                },
                parts: vec![GooglePart { text: m.content }],
            })
            .collect();

        let req = GoogleRequest {
            contents,
            generation_config: GoogleGenerationConfig {
                temperature: request.temperature,
                max_output_tokens: request.max_tokens,
            },
        };

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            request.model, self.api_key
        );

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&req)
            .send()
            .await?;

        let response: GoogleResponse = response.json().await?;

        Ok(ChatResponse {
            content: response
                .candidates
                .first()
                .and_then(|c| c.content.parts.first())
                .map(|p| p.text.clone())
                .unwrap_or_default(),
            model: request.model.clone(),
            tokens_used: response
                .usage_metadata
                .map(|u| u.total_token_count),
            finish_reason: response
                .candidates
                .first()
                .and_then(|c| c.finish_reason.clone()),
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
            "gemini-pro" => Some(ModelInfo {
                id: "gemini-pro".to_string(),
                display_name: "Gemini Pro".to_string(),
                context_window: 32768,
                supports_streaming: true,
                supports_images: false,
            }),
            "gemini-pro-vision" => Some(ModelInfo {
                id: "gemini-pro-vision".to_string(),
                display_name: "Gemini Pro Vision".to_string(),
                context_window: 16384,
                supports_streaming: true,
                supports_images: true,
            }),
            _ => None,
        }
    }

    fn list_models(&self) -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: "gemini-pro".to_string(),
                display_name: "Gemini Pro".to_string(),
                context_window: 32768,
                supports_streaming: true,
                supports_images: false,
            },
            ModelInfo {
                id: "gemini-pro-vision".to_string(),
                display_name: "Gemini Pro Vision".to_string(),
                context_window: 16384,
                supports_streaming: true,
                supports_images: true,
            },
        ]
    }
}

