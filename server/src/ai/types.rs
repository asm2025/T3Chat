use crate::db::models::MessageRole;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Model parameters for AI completion requests
/// Corresponds to the JSONB model_parameters field in chats/presets
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelParameters {
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f32>,
    pub top_k: Option<u32>,
    pub presence_penalty: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub stop_sequences: Option<Vec<String>>,
    pub reasoning_effort: Option<String>,
    #[serde(flatten)]
    pub extra: Option<Value>, // Provider-specific fields
}

/// Feature flags for provider-specific capabilities
/// Corresponds to the JSONB feature_flags field in chats/presets
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FeatureFlags {
    pub resend_files: Option<bool>,
    pub resend_images: Option<bool>,
    pub image_detail: Option<String>, // "low", "high", "auto"
    pub prompt_cache: Option<bool>,
    pub thinking: Option<bool>,
    pub thinking_budget: Option<u32>,
    #[serde(flatten)]
    pub extra: Option<Value>, // Provider-specific flags
}

/// Chat request with full LibreChat support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub parameters: ModelParameters,
    pub feature_flags: Option<FeatureFlags>,
    pub system_message: Option<String>,
    pub stream: bool,
}

/// Single chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Chat completion response (non-streaming)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
    pub model: String,
    pub usage: Option<TokenUsage>,
    pub finish_reason: Option<String>,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Streaming response chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponseChunk {
    pub delta: String,
    pub done: bool,
    pub model: Option<String>,
    pub finish_reason: Option<String>,
}

/// Model information/metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub display_name: String,
    pub context_window: u32,
    pub max_output_tokens: Option<u32>,
    pub supports_streaming: bool,
    pub supports_images: bool,
    pub supports_functions: bool,
    pub supports_vision: bool,
    pub cost_per_input_token: Option<f64>,
    pub cost_per_output_token: Option<f64>,
}
