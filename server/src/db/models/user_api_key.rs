use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::models::AiProvider;
use crate::db::schema::user_api_keys;

#[derive(
    Debug, Clone, PartialEq, Eq, Queryable, Selectable, Identifiable, Serialize, Deserialize,
)]
#[diesel(table_name = user_api_keys)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserApiKeyModel {
    pub id: Uuid,
    pub user_id: String,
    pub provider: AiProvider,
    pub encrypted_key: String,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = user_api_keys)]
pub struct NewUserApiKey {
    pub id: Uuid,
    pub user_id: String,
    pub provider: AiProvider,
    pub encrypted_key: String,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = user_api_keys)]
pub struct UpdateUserApiKey {
    pub encrypted_key: Option<String>,
    pub is_default: Option<bool>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserApiKeyDto {
    pub user_id: String,
    pub provider: AiProvider,
    pub encrypted_key: String,
    pub is_default: bool,
}

impl From<CreateUserApiKeyDto> for NewUserApiKey {
    fn from(dto: CreateUserApiKeyDto) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id: dto.user_id,
            provider: dto.provider,
            encrypted_key: dto.encrypted_key,
            is_default: dto.is_default,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserApiKeyDto {
    pub encrypted_key: Option<String>,
    pub is_default: Option<bool>,
}

impl From<UpdateUserApiKeyDto> for UpdateUserApiKey {
    fn from(dto: UpdateUserApiKeyDto) -> Self {
        Self {
            encrypted_key: dto.encrypted_key,
            is_default: dto.is_default,
            updated_at: Utc::now(),
        }
    }
}











