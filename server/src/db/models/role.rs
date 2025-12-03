use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// Note: This table is created by migration, but schema.rs needs to be regenerated
// For now, we'll use raw SQL queries or add the table definition manually

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}
