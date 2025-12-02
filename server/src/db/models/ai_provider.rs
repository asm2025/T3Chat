use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::db::schema::ai_providers;

/// AI Provider model - represents an AI provider configuration
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = ai_providers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AiProvider {
    pub id: Uuid,
    pub provider_id: String,  // openai, anthropic, google, etc.
    pub display_name: String,
    pub description: Option<String>,
    pub base_url: Option<String>,
    pub website_url: Option<String>,
    pub documentation_url: Option<String>,
    pub disabled: bool,
    pub is_active: bool,
    pub requires_api_key: bool,
    pub supports_streaming: bool,
    pub supports_images: bool,
    pub supports_functions: bool,
    pub supports_vision: bool,
    pub metadata: Option<JsonValue>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// New AI provider creation
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = ai_providers)]
pub struct NewAiProvider {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub provider_id: String,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation_url: Option<String>,
    #[serde(default)]
    pub disabled: bool,
    #[serde(default = "default_true")]
    pub is_active: bool,
    #[serde(default = "default_true")]
    pub requires_api_key: bool,
    #[serde(default = "default_true")]
    pub supports_streaming: bool,
    #[serde(default)]
    pub supports_images: bool,
    #[serde(default)]
    pub supports_functions: bool,
    #[serde(default)]
    pub supports_vision: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<JsonValue>,
}

fn default_true() -> bool {
    true
}

/// AI provider update
#[derive(Debug, Clone, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = ai_providers)]
pub struct UpdateAiProvider {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website_url: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation_url: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_api_key: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_streaming: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_images: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_functions: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_vision: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Option<JsonValue>>,
    pub updated_at: DateTime<Utc>,
}

impl AiProvider {
    /// Check if provider is active and enabled
    pub fn is_available(&self) -> bool {
        self.is_active && !self.disabled
    }
}

