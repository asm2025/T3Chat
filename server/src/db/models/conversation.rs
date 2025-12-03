use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::db::schema::{conversations, messages};

/// Conversation model
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = conversations)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Conversation {
    pub id: Uuid,
    pub conversation_id: String, // for API compatibility
    pub user_id: String,
    pub title: Option<String>,

    // Current provider/model (user's last selection, can change per message)
    pub endpoint: String, // openai, anthropic, google, custom, etc.
    pub model: String,
    pub model_label: Option<String>,

    // AI Parameters (JSONB - dynamic, varies by provider)
    pub model_parameters: Option<JsonValue>,

    // System/Instructions
    pub system_message: Option<String>,
    pub instructions: Option<String>,

    // Feature Flags (JSONB - optional boolean flags, varies by provider)
    pub feature_flags: Option<JsonValue>,

    // Agent/Assistant references (optional)
    pub agent_id: Option<Uuid>,
    pub assistant_id: Option<Uuid>,
    pub agent_options: Option<JsonValue>,

    // Metadata
    pub is_archived: Option<bool>,

    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// New conversation creation
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = conversations)]
pub struct NewConversation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub conversation_id: String,
    pub user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub endpoint: String,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_parameters: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feature_flags: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assistant_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_options: Option<JsonValue>,
}

/// Conversation update
#[derive(Debug, Clone, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = conversations)]
pub struct UpdateConversation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_label: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_parameters: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_message: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feature_flags: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<Option<Uuid>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assistant_id: Option<Option<Uuid>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_options: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_archived: Option<bool>,
    pub updated_at: DateTime<Utc>,
}

/// Message model
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = messages)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Message {
    pub id: Uuid,
    pub message_id: String, // for API compatibility
    pub conversation_id: Uuid,
    pub parent_message_id: Option<Uuid>,

    // Message basics
    pub role: String, // user, assistant, system, tool
    pub text: Option<String>,
    pub is_created_by_user: bool,

    // AI/Model info (stored for historical accuracy)
    pub model: Option<String>,
    pub endpoint: Option<String>,

    // Content (for multimodal messages)
    pub content: Option<JsonValue>,

    // Completion info
    pub token_count: Option<i32>,
    pub finish_reason: Option<String>,
    pub error: Option<bool>,

    // File attachments
    pub file_ids: Option<Vec<Option<Uuid>>>,

    // Tool/Plugin data
    pub tool_call_id: Option<String>,
    pub plugin_data: Option<JsonValue>,

    // Metadata
    pub thread_id: Option<String>,

    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// New message creation
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = messages)]
pub struct NewMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub message_id: String,
    pub conversation_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_message_id: Option<Uuid>,
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    pub is_created_by_user: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_ids: Option<Vec<Option<Uuid>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin_data: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
}

/// Message update
#[derive(Debug, Clone, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = messages)]
pub struct UpdateMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<bool>,
    pub updated_at: DateTime<Utc>,
}

// DTOs for message operations (for backward compatibility with old message API)
use crate::db::models::message::{CreateMessageDto, UpdateMessageDto as OldUpdateMessageDto};

impl From<CreateMessageDto> for NewMessage {
    fn from(dto: CreateMessageDto) -> Self {
        Self {
            id: Some(uuid::Uuid::new_v4()),
            message_id: uuid::Uuid::new_v4().to_string(),
            conversation_id: dto.chat_id,
            parent_message_id: dto.parent_message_id,
            role: dto.role.as_str().to_string(),
            text: Some(dto.content),
            is_created_by_user: dto.role == crate::db::models::message::MessageRole::User,
            model: None,
            endpoint: None,
            content: dto.metadata,
            token_count: None,
            finish_reason: None,
            file_ids: None,
            tool_call_id: None,
            plugin_data: None,
            thread_id: None,
        }
    }
}

impl From<OldUpdateMessageDto> for UpdateMessage {
    fn from(dto: OldUpdateMessageDto) -> Self {
        Self {
            text: dto.content.map(Some),
            content: dto.metadata,
            token_count: None,
            finish_reason: None,
            error: None,
            updated_at: chrono::Utc::now(),
        }
    }
}
