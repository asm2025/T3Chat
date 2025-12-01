use diesel::sql_types::Text;
use diesel::{AsExpression, FromSqlRow};
use serde::{Deserialize, Serialize};

// Core models
pub mod user;
pub mod ai_model;
pub mod user_api_key;

// Conversation & messaging
pub mod conversation;
pub mod preset;
pub mod file;

// Agent system
pub mod agent;

// Tools & actions
pub mod tool;

// Organization
pub mod tag;
pub mod project;

// Billing & tracking
pub mod transaction;

// Sharing
pub mod shared_link;

// Legacy models (keeping for backward compatibility during migration)
pub mod chat;
pub mod feature;
pub mod message;

// Re-export all models for convenience
pub use user::*;
pub use ai_model::*;
pub use user_api_key::*;
pub use conversation::*;
pub use preset::*;
pub use file::*;
pub use agent::*;
pub use tool::*;
pub use tag::*;
pub use project::*;
pub use transaction::*;

// Legacy re-exports - only export specific types to avoid conflicts
pub use chat::{ChatModel, CreateChatDto, UpdateChatDto};
pub use feature::*;
// Note: message::NewMessage and message::UpdateMessage conflict with conversation types
// Use conversation::NewMessage and conversation::UpdateMessage for database operations
pub use message::{MessageRole, CreateMessageDto, UpdateMessageDto};

// Type aliases for compatibility
pub type UserModel = User;
pub type UserApiKeyModel = UserApiKey;
pub type AiModelModel = AiModel;
pub type ConversationModel = Conversation;
pub type MessageModel = Message;

// AI Provider enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, AsExpression, FromSqlRow)]
#[diesel(sql_type = Text)]
pub enum AiProvider {
    OpenAI,
    Anthropic,
    Google,
    DeepSeek,
    Ollama,
}

impl AiProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            AiProvider::OpenAI => "openai",
            AiProvider::Anthropic => "anthropic",
            AiProvider::Google => "google",
            AiProvider::DeepSeek => "deepseek",
            AiProvider::Ollama => "ollama",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "openai" => Some(AiProvider::OpenAI),
            "anthropic" => Some(AiProvider::Anthropic),
            "google" => Some(AiProvider::Google),
            "deepseek" => Some(AiProvider::DeepSeek),
            "ollama" => Some(AiProvider::Ollama),
            _ => None,
        }
    }
}

impl<DB> diesel::serialize::ToSql<Text, DB> for AiProvider
where
    DB: diesel::backend::Backend,
    str: diesel::serialize::ToSql<Text, DB>,
{
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, DB>,
    ) -> diesel::serialize::Result {
        self.as_str().to_sql(out)
    }
}

impl<DB> diesel::deserialize::FromSql<Text, DB> for AiProvider
where
    DB: diesel::backend::Backend,
    String: diesel::deserialize::FromSql<Text, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let s = String::from_sql(bytes)?;
        AiProvider::from_str(&s).ok_or_else(|| format!("Invalid AiProvider value: {}", s).into())
    }
}

// DTOs for user operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserDto {
    pub id: String,
    pub email: String,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    pub provider: Option<String>,
}

impl From<CreateUserDto> for NewUser {
    fn from(dto: CreateUserDto) -> Self {
        Self {
            id: dto.id,
            email: dto.email,
            email_verified: dto.email_verified,
            name: dto.name,
            username: dto.username,
            avatar_url: dto.avatar_url,
            provider: dto.provider.unwrap_or_else(|| "firebase".to_string()),
            role: None,
            preferences: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserDto {
    pub name: Option<String>,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    pub preferences: Option<serde_json::Value>,
}

impl From<UpdateUserDto> for UpdateUser {
    fn from(dto: UpdateUserDto) -> Self {
        Self {
            email: None,
            email_verified: None,
            name: dto.name,
            username: dto.username,
            avatar_url: dto.avatar_url,
            role: None,
            preferences: dto.preferences,
            terms_accepted: None,
            terms_accepted_at: None,
            updated_at: chrono::Utc::now(),
        }
    }
}

// DTOs for user API key operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserApiKeyDto {
    pub user_id: String,
    pub provider: AiProvider,
    pub encrypted_key: String,
    pub key_name: Option<String>,
    pub is_default: Option<bool>,
}

impl From<CreateUserApiKeyDto> for NewUserApiKey {
    fn from(dto: CreateUserApiKeyDto) -> Self {
        Self {
            id: None,
            user_id: dto.user_id,
            provider: dto.provider.as_str().to_string(),
            encrypted_key: dto.encrypted_key,
            key_name: dto.key_name,
            is_default: dto.is_default,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserApiKeyDto {
    pub encrypted_key: Option<String>,
    pub key_name: Option<String>,
    pub is_default: Option<bool>,
}

impl From<UpdateUserApiKeyDto> for UpdateUserApiKey {
    fn from(dto: UpdateUserApiKeyDto) -> Self {
        Self {
            encrypted_key: dto.encrypted_key,
            key_name: dto.key_name.map(Some),
            is_default: dto.is_default,
            last_used_at: None,
            updated_at: chrono::Utc::now(),
        }
    }
}
