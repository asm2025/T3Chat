use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::db::schema::presets;

/// Preset model - saved chat configurations
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = presets)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Preset {
    pub id: Uuid,
    pub preset_id: String, // for API compatibility
    pub user_id: String,
    pub title: String,
    pub is_default: Option<bool>,
    pub order_index: Option<i32>,

    // Provider/Model
    pub endpoint: String,
    pub model: String,
    pub model_label: Option<String>,

    // AI Parameters (same structure as chats)
    pub model_parameters: Option<JsonValue>,

    // System/Instructions
    pub system_message: Option<String>,
    pub instructions: Option<String>,

    // Feature Flags
    pub feature_flags: Option<JsonValue>,

    // Agent reference (optional)
    pub agent_id: Option<Uuid>,
    pub agent_options: Option<JsonValue>,

    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// New preset creation
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = presets)]
pub struct NewPreset {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub preset_id: String,
    pub user_id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_default: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_index: Option<i32>,
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
    pub agent_options: Option<JsonValue>,
}

/// Preset update
#[derive(Debug, Clone, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = presets)]
pub struct UpdatePreset {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_default: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_index: Option<i32>,
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
    pub agent_options: Option<JsonValue>,
    pub updated_at: DateTime<Utc>,
}
