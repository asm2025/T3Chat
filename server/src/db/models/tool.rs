use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::db::schema::{actions, agent_actions, agent_tools, assistant_tools, tool_calls, tools};

/// Tool model
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = tools)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Tool {
    pub id: Uuid,
    pub name: String, // web_search, code_interpreter, dalle, retrieval, etc.
    pub display_name: String,
    pub description: Option<String>,
    pub tool_type: String, // system, plugin, function, action
    pub icon_url: Option<String>,
    pub is_active: Option<bool>,
    pub is_system: Option<bool>,
    pub configuration_schema: Option<JsonValue>,

    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// New tool creation
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = tools)]
pub struct NewTool {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub name: String,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub tool_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_system: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration_schema: Option<JsonValue>,
}

/// Tool update
#[derive(Debug, Clone, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = tools)]
pub struct UpdateTool {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration_schema: Option<JsonValue>,
    pub updated_at: DateTime<Utc>,
}

/// Tool call model (execution logs)
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = tool_calls)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ToolCall {
    pub id: Uuid,
    pub message_id: Uuid,

    // Tool info
    pub tool_call_id: String,
    pub tool_name: String,
    pub tool_type: Option<String>, // function, code_interpreter, retrieval, web_search

    // Execution
    pub arguments: Option<JsonValue>,
    pub result: Option<JsonValue>,
    pub status: Option<String>, // pending, running, completed, failed
    pub error_message: Option<String>,

    // Output files
    pub output_file_ids: Option<Vec<Option<Uuid>>>,

    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// New tool call creation
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = tool_calls)]
pub struct NewToolCall {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub message_id: Uuid,
    pub tool_call_id: String,
    pub tool_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// Tool call update
#[derive(Debug, Clone, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = tool_calls)]
pub struct UpdateToolCall {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_file_ids: Option<Vec<Option<Uuid>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

/// Action model (custom tools/plugins)
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = actions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Action {
    pub id: Uuid,
    pub action_id: String,
    pub user_id: String,

    // Basic info
    pub name: String,
    pub description: Option<String>,
    pub action_type: Option<String>, // openapi, function, webhook

    // Configuration
    pub domain: Option<String>,
    pub endpoint_url: Option<String>,
    pub settings: Option<JsonValue>,

    // Authentication
    pub auth_type: Option<String>, // none, api_key, oauth, bearer
    pub auth_config: Option<JsonValue>,

    // OpenAPI spec
    pub openapi_spec: Option<String>,

    // Privacy
    pub privacy_policy_url: Option<String>,

    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// New action creation
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = actions)]
pub struct NewAction {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub action_id: String,
    pub user_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_config: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub openapi_spec: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privacy_policy_url: Option<String>,
}

/// Action update
#[derive(Debug, Clone, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = actions)]
pub struct UpdateAction {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_type: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint_url: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_type: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_config: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub openapi_spec: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privacy_policy_url: Option<Option<String>>,
    pub updated_at: DateTime<Utc>,
}

// Junction table models

/// Agent-Tool relationship
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = agent_tools)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AgentTool {
    pub agent_id: Uuid,
    pub tool_id: Uuid,
    pub configuration: Option<JsonValue>,
    pub is_enabled: Option<bool>,
    pub created_at: DateTime<Utc>,
}

/// New agent-tool relationship
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = agent_tools)]
pub struct NewAgentTool {
    pub agent_id: Uuid,
    pub tool_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_enabled: Option<bool>,
}

/// Assistant-Tool relationship
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = assistant_tools)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AssistantTool {
    pub assistant_id: Uuid,
    pub tool_id: Uuid,
    pub configuration: Option<JsonValue>,
    pub created_at: DateTime<Utc>,
}

/// New assistant-tool relationship
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = assistant_tools)]
pub struct NewAssistantTool {
    pub assistant_id: Uuid,
    pub tool_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration: Option<JsonValue>,
}

/// Agent-Action relationship
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = agent_actions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AgentAction {
    pub agent_id: Uuid,
    pub action_id: Uuid,
    pub is_enabled: Option<bool>,
    pub created_at: DateTime<Utc>,
}

/// New agent-action relationship
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = agent_actions)]
pub struct NewAgentAction {
    pub agent_id: Uuid,
    pub action_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_enabled: Option<bool>,
}
