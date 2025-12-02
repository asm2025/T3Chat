use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRole {
    pub user_id: String,
    pub role_name: String,
    pub assigned_at: DateTime<Utc>,
    pub assigned_by: Option<String>,
}

