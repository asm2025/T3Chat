use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::db::schema::users;

/// User model - represents a user in the system
/// Field order must match schema.rs exactly
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: String, // OIDC subject (sub) claim
    pub email: String,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    pub provider: String,     // oidc, google, local, etc.
    pub role: Option<String>, // user, admin, moderator
    pub password_hash: Option<String>,
    pub two_factor_enabled: Option<bool>,
    pub totp_secret: Option<String>, // encrypted
    pub preferences: Option<JsonValue>,
    pub terms_accepted: Option<bool>,
    pub terms_accepted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub normalized_email: String,
    pub normalized_username: Option<String>,
    pub disabled: bool,
    pub locked_out: bool,
    pub lockout_end: Option<DateTime<Utc>>,
    pub access_failed_count: i32,
    pub password_changed_at: Option<DateTime<Utc>>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub login_count: i32,
}

/// New user creation
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub id: String, // OIDC subject (sub) claim
    pub email: String,
    pub normalized_email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_verified: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normalized_username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(default = "default_provider")]
    pub provider: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(default)]
    pub disabled: bool,
    #[serde(default)]
    pub locked_out: bool,
    #[serde(default)]
    pub access_failed_count: i32,
    #[serde(default)]
    pub login_count: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferences: Option<JsonValue>,
}

fn default_provider() -> String {
    "oidc".to_string()
}

/// User update
#[derive(Debug, Clone, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = users)]
pub struct UpdateUser {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_verified: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferences: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terms_accepted: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terms_accepted_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    /// Check if user is an admin
    pub fn is_admin(&self) -> bool {
        self.role.as_deref() == Some("admin")
    }

    /// Check if user is a moderator
    pub fn is_moderator(&self) -> bool {
        matches!(self.role.as_deref(), Some("admin") | Some("moderator"))
    }

    /// Check if user has accepted terms
    pub fn has_accepted_terms(&self) -> bool {
        self.terms_accepted.unwrap_or(false)
    }
}
