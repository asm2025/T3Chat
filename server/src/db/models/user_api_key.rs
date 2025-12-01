use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::schema::user_api_keys;

/// User API key model (encrypted storage)
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = user_api_keys)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserApiKey {
    pub id: Uuid,
    pub user_id: String,
    pub provider: String,  // openai, anthropic, google, custom
    pub encrypted_key: String,  // AES-256-GCM encrypted
    pub key_name: Option<String>,  // user-friendly name
    pub is_default: Option<bool>,
    
    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

/// New user API key creation
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = user_api_keys)]
pub struct NewUserApiKey {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub user_id: String,
    pub provider: String,
    pub encrypted_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_default: Option<bool>,
}

/// User API key update
#[derive(Debug, Clone, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = user_api_keys)]
pub struct UpdateUserApiKey {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypted_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_name: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_default: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

impl UserApiKey {
    /// Check if this is the default key for the provider
    pub fn is_default(&self) -> bool {
        self.is_default.unwrap_or(false)
    }

    /// Get display name (key_name or provider)
    pub fn display_name(&self) -> String {
        self.key_name
            .clone()
            .unwrap_or_else(|| format!("{} Key", self.provider))
    }
}
