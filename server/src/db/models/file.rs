use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::db::schema::files;

/// File model
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = files)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[serde(rename_all = "camelCase")]
pub struct File {
    pub id: Uuid,
    pub file_id: String, // for API compatibility
    pub user_id: String,
    pub chat_id: Option<Uuid>,

    // File info
    pub filename: String,
    pub filepath: String, // relative path in storage
    pub mime_type: String,
    pub size_bytes: i64,

    // File type categorization
    pub file_type: String, // image, document, audio, video, other

    // Content (for text files / OCR)
    pub text_content: Option<String>,
    pub is_embedded: Option<bool>, // vector embeddings created

    // Image-specific
    pub width: Option<i32>,
    pub height: Option<i32>,

    // Metadata
    pub source: Option<String>, // upload, url, generated
    pub metadata: Option<JsonValue>,

    // Usage tracking
    pub usage_count: Option<i32>,
    pub last_used_at: Option<DateTime<Utc>>,

    // Temporary files
    pub is_temporary: Option<bool>,
    pub expires_at: Option<DateTime<Utc>>,

    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// New file creation
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = files)]
pub struct NewFile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub file_id: String,
    pub user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chat_id: Option<Uuid>,
    pub filename: String,
    pub filepath: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub file_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_temporary: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}

/// File update
#[derive(Debug, Clone, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = files)]
pub struct UpdateFile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chat_id: Option<Option<Uuid>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_content: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_embedded: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

impl File {
    /// Check if file is an image
    pub fn is_image(&self) -> bool {
        self.file_type == "image"
    }

    /// Check if file is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }
}
