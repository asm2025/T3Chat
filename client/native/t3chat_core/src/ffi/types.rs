use serde::{Deserialize, Serialize};

// FFI-safe types that mirror domain models
// These are used for communication between Rust and Dart

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FfiUser {
    pub id: String,
    pub email: String,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    pub roles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FfiUserAndToken {
    pub user: FfiUser,
    pub token: String,
    pub expires_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FfiChat {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub model_provider: String,
    pub model_id: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FfiMessage {
    pub id: String,
    pub chat_id: String,
    pub role: String,
    pub content: String,
    pub metadata: Option<String>, // JSON string
    pub parent_message_id: Option<String>,
    pub sequence_number: i32,
    pub created_at: String,
    pub tokens_used: Option<i32>,
    pub model_used: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FfiChatWithMessages {
    pub chat: FfiChat,
    pub messages: Vec<FfiMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FfiChatListResponse {
    pub data: Vec<FfiChat>,
    pub total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FfiChatChunk {
    pub delta: String,
    pub done: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FfiAuthConfig {
    pub oidc_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FfiError {
    Network(String),
    Auth(String),
    NotFound(String),
    Server(String),
    Unknown(String),
}

impl From<crate::domain::Error> for FfiError {
    fn from(err: crate::domain::Error) -> Self {
        match err {
            crate::domain::Error::Network(msg) => FfiError::Network(msg),
            crate::domain::Error::Auth(msg) => FfiError::Auth(msg),
            crate::domain::Error::NotFound(msg) => FfiError::NotFound(msg),
            crate::domain::Error::Server(msg) => FfiError::Server(msg),
            crate::domain::Error::Storage(msg) => FfiError::Unknown(format!("Storage: {}", msg)),
            crate::domain::Error::Unknown(msg) => FfiError::Unknown(msg),
        }
    }
}

// Conversion helpers
impl From<crate::domain::User> for FfiUser {
    fn from(user: crate::domain::User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            email_verified: user.email_verified,
            name: user.name,
            username: user.username,
            avatar_url: user.avatar_url,
            roles: user.roles,
        }
    }
}

impl From<crate::domain::Chat> for FfiChat {
    fn from(chat: crate::domain::Chat) -> Self {
        Self {
            id: chat.id,
            user_id: chat.user_id,
            title: chat.title,
            model_provider: chat.model_provider,
            model_id: chat.model_id,
            created_at: chat.created_at,
            updated_at: chat.updated_at,
        }
    }
}

impl From<crate::domain::Message> for FfiMessage {
    fn from(msg: crate::domain::Message) -> Self {
        Self {
            id: msg.id,
            chat_id: msg.chat_id,
            role: msg.role,
            content: msg.content,
            metadata: msg.metadata.map(|v| v.to_string()),
            parent_message_id: msg.parent_message_id,
            sequence_number: msg.sequence_number,
            created_at: msg.created_at,
            tokens_used: msg.tokens_used,
            model_used: msg.model_used,
        }
    }
}

