use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: String,
    pub email: String,
    #[serde(alias = "email_verified")]
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub username: Option<String>,
    #[serde(alias = "avatar_url")]
    pub avatar_url: Option<String>,
    pub roles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserAndToken {
    pub user: User,
    pub token: String,
    #[serde(alias = "expires_at")]
    pub expires_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Chat {
    pub id: String, // UUID as string
    #[serde(alias = "user_id")]
    pub user_id: String,
    pub title: String,
    #[serde(alias = "model_provider")]
    pub model_provider: String,
    #[serde(alias = "model_id")]
    pub model_id: String,
    #[serde(alias = "created_at")]
    pub created_at: String, // ISO 8601
    #[serde(alias = "updated_at")]
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: String, // UUID as string
    #[serde(alias = "chat_id")]
    pub chat_id: String,
    pub role: String,
    pub content: String,
    pub metadata: Option<serde_json::Value>,
    #[serde(alias = "parent_message_id")]
    pub parent_message_id: Option<String>,
    #[serde(alias = "sequence_number")]
    pub sequence_number: i32,
    #[serde(alias = "created_at")]
    pub created_at: String,
    #[serde(alias = "tokens_used")]
    pub tokens_used: Option<i32>,
    #[serde(alias = "model_used")]
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
#[serde(rename_all = "camelCase")]
pub struct ChatCompletionResponse {
    pub content: String,
    pub model: String,
    pub usage: Option<TokenUsage>,
    #[serde(alias = "finish_reason")]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsage {
    #[serde(alias = "prompt_tokens")]
    pub prompt_tokens: u32,
    #[serde(alias = "completion_tokens")]
    pub completion_tokens: u32,
    #[serde(alias = "total_tokens")]
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthConfig {
    #[serde(alias = "oidc_enabled")]
    pub oidc_enabled: bool,
}

