use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::schema::shared_links;

/// Shared link model (conversation sharing)
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = shared_links)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SharedLink {
    pub id: Uuid,
    pub share_id: String,  // public share ID
    pub conversation_id: Uuid,
    pub user_id: String,
    
    // Sharing settings
    pub is_public: Option<bool>,
    pub is_anonymous: Option<bool>,  // hide user info
    pub title: Option<String>,  // custom title for shared link
    
    // Access control
    pub password_hash: Option<String>,
    pub max_views: Option<i32>,
    pub view_count: Option<i32>,
    pub expires_at: Option<DateTime<Utc>>,
    
    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_viewed_at: Option<DateTime<Utc>>,
}

/// New shared link creation
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = shared_links)]
pub struct NewSharedLink {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub share_id: String,
    pub conversation_id: Uuid,
    pub user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_public: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_anonymous: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_views: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}

/// Shared link update
#[derive(Debug, Clone, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = shared_links)]
pub struct UpdateSharedLink {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_public: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_anonymous: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_hash: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_views: Option<Option<i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<Option<DateTime<Utc>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_viewed_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

impl SharedLink {
    /// Check if link has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Check if link has reached max views
    pub fn has_reached_max_views(&self) -> bool {
        if let (Some(max_views), Some(view_count)) = (self.max_views, self.view_count) {
            view_count >= max_views
        } else {
            false
        }
    }

    /// Check if link is accessible
    pub fn is_accessible(&self) -> bool {
        !self.is_expired() && !self.has_reached_max_views()
    }
}

