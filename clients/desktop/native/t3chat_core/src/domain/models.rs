use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    #[serde(rename = "emailVerified")]
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub username: Option<String>,
    #[serde(rename = "avatarUrl")]
    pub avatar_url: Option<String>,
    pub roles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAndToken {
    pub user: User,
    pub token: String,
    #[serde(rename = "expiresAt")]
    pub expires_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chat {
    pub id: String, // UUID as string
    #[serde(rename = "userId")]
    pub user_id: String,
    pub title: String,
    #[serde(rename = "modelProvider")]
    pub model_provider: String,
    #[serde(rename = "modelId")]
    pub model_id: String,
    #[serde(rename = "createdAt")]
    pub created_at: String, // ISO 8601
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String, // UUID as string
    #[serde(rename = "chatId")]
    pub chat_id: String,
    pub role: String,
    pub content: String,
    pub metadata: Option<serde_json::Value>,
    #[serde(rename = "parentMessageId")]
    pub parent_message_id: Option<String>,
    #[serde(rename = "sequenceNumber")]
    pub sequence_number: i32,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "tokensUsed")]
    pub tokens_used: Option<i32>,
    #[serde(rename = "modelUsed")]
    pub model_used: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatWithMessages {
    pub chat: Chat,
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatListResponse {
    pub data: Vec<Chat>,
    pub total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChunk {
    pub delta: String,
    pub done: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    pub content: String,
    pub model: String,
    pub usage: Option<TokenUsage>,
    #[serde(rename = "finishReason")]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    #[serde(rename = "promptTokens")]
    pub prompt_tokens: u32,
    #[serde(rename = "completionTokens")]
    pub completion_tokens: u32,
    #[serde(rename = "totalTokens")]
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    #[serde(rename = "oidcEnabled")]
    pub oidc_enabled: bool,
}

