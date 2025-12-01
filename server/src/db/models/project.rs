use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::schema::{projects, prompt_groups, prompts, project_agents, agent_hierarchy};

/// Project model
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = projects)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: String,
    
    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// New project creation
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = projects)]
pub struct NewProject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub owner_id: String,
}

/// Project update
#[derive(Debug, Clone, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = projects)]
pub struct UpdateProject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Option<String>>,
    pub updated_at: DateTime<Utc>,
}

/// Prompt group model
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = prompt_groups)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PromptGroup {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub author_id: String,
    pub project_id: Option<Uuid>,
    
    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// New prompt group creation
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = prompt_groups)]
pub struct NewPromptGroup {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub author_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<Uuid>,
}

/// Prompt group update
#[derive(Debug, Clone, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = prompt_groups)]
pub struct UpdatePromptGroup {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<Option<Uuid>>,
    pub updated_at: DateTime<Utc>,
}

/// Prompt model
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = prompts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Prompt {
    pub id: Uuid,
    pub group_id: Uuid,
    pub title: Option<String>,
    pub prompt_text: String,
    pub prompt_type: String,  // system, user, template
    pub variables: Option<Vec<Option<String>>>,  // template variables
    pub order_index: Option<i32>,
    
    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// New prompt creation
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = prompts)]
pub struct NewPrompt {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub group_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub prompt_text: String,
    pub prompt_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<Vec<Option<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_index: Option<i32>,
}

/// Prompt update
#[derive(Debug, Clone, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = prompts)]
pub struct UpdatePrompt {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<Vec<Option<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_index: Option<i32>,
    pub updated_at: DateTime<Utc>,
}

// Junction table models

/// Project-Agent relationship
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = project_agents)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ProjectAgent {
    pub project_id: Uuid,
    pub agent_id: Uuid,
    pub role: Option<String>,
    pub order_index: Option<i32>,
    pub created_at: DateTime<Utc>,
}

/// New project-agent relationship
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = project_agents)]
pub struct NewProjectAgent {
    pub project_id: Uuid,
    pub agent_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_index: Option<i32>,
}

/// Agent hierarchy (parent-child relationship)
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = agent_hierarchy)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AgentHierarchy {
    pub parent_agent_id: Uuid,
    pub sub_agent_id: Uuid,
    pub order_index: Option<i32>,
    pub created_at: DateTime<Utc>,
}

/// New agent hierarchy relationship
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = agent_hierarchy)]
pub struct NewAgentHierarchy {
    pub parent_agent_id: Uuid,
    pub sub_agent_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_index: Option<i32>,
}

