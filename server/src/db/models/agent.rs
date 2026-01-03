use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;
use utoipa::ToSchema;

use crate::db::schema::{agent_chat_starters, agents, assistant_chat_starters, assistants};

/// Agent model
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize, ToSchema)]
#[diesel(table_name = agents)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Agent {
    pub id: Uuid,
    pub agent_id: String, // for API compatibility
    pub author_id: String,

    // Basic info
    pub name: String,
    pub description: Option<String>,
    pub instructions: Option<String>,

    // Avatar
    pub avatar_filepath: Option<String>,
    pub avatar_source: Option<String>,

    // Model configuration
    pub provider: String,
    pub model: String,
    pub model_parameters: Option<JsonValue>,

    // Behavior
    pub access_level: Option<i32>, // 0=private, 1=shared, 2=public
    pub recursion_limit: Option<i32>,
    pub hide_sequential_outputs: Option<bool>,
    pub end_after_tools: Option<bool>,
    pub is_collaborative: Option<bool>,

    // Tool resources (provider-specific configuration)
    pub tool_resources: Option<JsonValue>,

    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// New agent creation
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = agents)]
pub struct NewAgent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub agent_id: String,
    pub author_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_filepath: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_source: Option<String>,
    pub provider: String,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_parameters: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_level: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recursion_limit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hide_sequential_outputs: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_after_tools: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_collaborative: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_resources: Option<JsonValue>,
}

/// Agent update
#[derive(Debug, Clone, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = agents)]
pub struct UpdateAgent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_filepath: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_source: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_parameters: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_level: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recursion_limit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hide_sequential_outputs: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_after_tools: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_collaborative: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_resources: Option<JsonValue>,
    pub updated_at: DateTime<Utc>,
}

/// Assistant model (OpenAI Assistants API compatibility)
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = assistants)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Assistant {
    pub id: Uuid,
    pub assistant_id: String, // OpenAI assistant ID
    pub user_id: String,

    // Basic info
    pub name: Option<String>,
    pub description: Option<String>,
    pub instructions: Option<String>,

    // Avatar
    pub avatar_filepath: Option<String>,
    pub avatar_source: Option<String>,

    // Configuration
    pub model: String,
    pub tools: Option<JsonValue>, // OpenAI tools format
    pub file_ids: Option<Vec<Option<Uuid>>>,

    // Behavior
    pub access_level: Option<i32>,
    pub append_current_datetime: Option<bool>,

    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// New assistant creation
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = assistants)]
pub struct NewAssistant {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub assistant_id: String,
    pub user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_filepath: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_source: Option<String>,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_ids: Option<Vec<Option<Uuid>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_level: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub append_current_datetime: Option<bool>,
}

/// Assistant update
#[derive(Debug, Clone, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = assistants)]
pub struct UpdateAssistant {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_filepath: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_source: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_ids: Option<Vec<Option<Uuid>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_level: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub append_current_datetime: Option<bool>,
    pub updated_at: DateTime<Utc>,
}

/// Agent chat starter
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize, ToSchema)]
#[diesel(table_name = agent_chat_starters)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AgentChatStarter {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub text: String,
    pub order_index: Option<i32>,
    pub created_at: DateTime<Utc>,
}

/// New agent chat starter
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = agent_chat_starters)]
pub struct NewAgentChatStarter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub agent_id: Uuid,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_index: Option<i32>,
}

/// Assistant chat starter
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = assistant_chat_starters)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AssistantChatStarter {
    pub id: Uuid,
    pub assistant_id: Uuid,
    pub text: String,
    pub order_index: Option<i32>,
    pub created_at: DateTime<Utc>,
}

/// New assistant chat starter
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = assistant_chat_starters)]
pub struct NewAssistantChatStarter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub assistant_id: Uuid,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_index: Option<i32>,
}
