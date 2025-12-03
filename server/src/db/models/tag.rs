use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::schema::{conversation_tags_map, tags};

/// Tag model
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = tags)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Tag {
    pub id: Uuid,
    pub user_id: String,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>, // hex color for UI
    pub position: Option<i32>,

    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// New tag creation
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = tags)]
pub struct NewTag {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub user_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i32>,
}

/// Tag update
#[derive(Debug, Clone, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = tags)]
pub struct UpdateTag {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i32>,
    pub updated_at: DateTime<Utc>,
}

/// Conversation-Tag relationship (junction table)
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = conversation_tags_map)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ConversationTag {
    pub conversation_id: Uuid,
    pub tag_id: Uuid,
    pub created_at: DateTime<Utc>,
}

/// New conversation-tag relationship
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = conversation_tags_map)]
pub struct NewConversationTag {
    pub conversation_id: Uuid,
    pub tag_id: Uuid,
}
