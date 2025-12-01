use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::models::{AiProvider, Conversation};

// Legacy ChatModel - plain struct for API compatibility
// Note: No longer directly queryable from database (chats table doesn't exist)
// Use Conversation model for database operations and convert to ChatModel for API responses
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatModel {
    pub id: Uuid,
    pub user_id: String,
    pub title: String,
    pub model_provider: AiProvider,
    pub model_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// Convert from Conversation to ChatModel for API compatibility
impl From<Conversation> for ChatModel {
    fn from(conv: Conversation) -> Self {
        // Parse provider from endpoint string
        let model_provider = AiProvider::from_str(&conv.endpoint)
            .unwrap_or(AiProvider::OpenAI);
        
        Self {
            id: conv.id,
            user_id: conv.user_id,
            title: conv.title.unwrap_or_else(|| "New Chat".to_string()),
            model_provider,
            model_id: conv.model,
            created_at: conv.created_at,
            updated_at: conv.updated_at,
            deleted_at: None,  // Conversations use is_archived instead
        }
    }
}

// Legacy NewChat - not used for database inserts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewChat {
    pub id: Uuid,
    pub user_id: String,
    pub title: String,
    pub model_provider: AiProvider,
    pub model_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// Legacy UpdateChat - not used for database updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateChat {
    pub title: Option<String>,
    pub model_provider: Option<AiProvider>,
    pub model_id: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateChatDto {
    pub user_id: String,
    pub title: String,
    pub model_provider: AiProvider,
    pub model_id: String,
}

impl From<CreateChatDto> for NewChat {
    fn from(dto: CreateChatDto) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id: dto.user_id,
            title: dto.title,
            model_provider: dto.model_provider,
            model_id: dto.model_id,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateChatDto {
    pub title: Option<String>,
    pub model_provider: Option<AiProvider>,
    pub model_id: Option<String>,
}

impl From<UpdateChatDto> for UpdateChat {
    fn from(dto: UpdateChatDto) -> Self {
        Self {
            title: dto.title,
            model_provider: dto.model_provider,
            model_id: dto.model_id,
            updated_at: Utc::now(),
        }
    }
}

























