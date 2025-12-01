use rust_decimal::Decimal;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::schema::ai_models;

/// AI Model reference table (metadata/configuration)
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = ai_models)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AiModel {
    pub id: Uuid,
    pub provider: String,  // openai, anthropic, google, custom
    pub model_id: String,  // gpt-4-turbo, claude-3-opus, gemini-pro, etc.
    pub display_name: String,
    pub description: Option<String>,
    
    // Capabilities
    pub context_window: i32,
    pub max_output_tokens: Option<i32>,
    pub supports_streaming: Option<bool>,
    pub supports_images: Option<bool>,
    pub supports_functions: Option<bool>,
    pub supports_vision: Option<bool>,
    
    // Pricing (per 1M tokens)
    pub cost_per_input_token: Option<Decimal>,
    pub cost_per_output_token: Option<Decimal>,
    
    // Status
    pub is_active: Option<bool>,
    pub deprecated_at: Option<DateTime<Utc>>,
    
    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// New AI model creation
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = ai_models)]
pub struct NewAiModel {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub provider: String,
    pub model_id: String,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub context_window: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_streaming: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_images: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_functions: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_vision: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_per_input_token: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_per_output_token: Option<Decimal>,
}

/// AI model update
#[derive(Debug, Clone, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = ai_models)]
pub struct UpdateAiModel {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_window: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<Option<i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_streaming: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_images: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_functions: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_vision: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_per_input_token: Option<Option<Decimal>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_per_output_token: Option<Option<Decimal>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated_at: Option<Option<DateTime<Utc>>>,
    pub updated_at: DateTime<Utc>,
}

impl AiModel {
    /// Check if model is active
    pub fn is_active(&self) -> bool {
        self.is_active.unwrap_or(true) && self.deprecated_at.is_none()
    }

    /// Check if model supports streaming
    pub fn supports_streaming(&self) -> bool {
        self.supports_streaming.unwrap_or(true)
    }

    /// Get full model identifier
    pub fn full_identifier(&self) -> String {
        format!("{}:{}", self.provider, self.model_id)
    }
}
