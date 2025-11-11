use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::models::AiProvider;
use crate::db::schema::chats;

#[derive(
    Debug, Clone, PartialEq, Eq, Queryable, Selectable, Identifiable, Serialize, Deserialize,
)]
#[diesel(table_name = chats)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ChatModel {
    pub id: Uuid,
    pub user_id: String,
    pub title: String,
    pub model_provider: AiProvider,
    pub model_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = chats)]
pub struct NewChat {
    pub id: Uuid,
    pub user_id: String,
    pub title: String,
    pub model_provider: AiProvider,
    pub model_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = chats)]
pub struct UpdateChat {
    pub title: Option<String>,
    pub model_provider: Option<AiProvider>,
    pub model_id: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateChatDto {
    pub user_id: String,
    pub title: String,
    pub model_provider: AiProvider,
    pub model_id: String,
}

impl From<CreateChatDto> for NewChat {
    fn from(dto: CreateChatDto) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id: dto.user_id,
            title: dto.title,
            model_provider: dto.model_provider,
            model_id: dto.model_id,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateChatDto {
    pub title: Option<String>,
    pub model_provider: Option<AiProvider>,
    pub model_id: Option<String>,
}

impl From<UpdateChatDto> for UpdateChat {
    fn from(dto: UpdateChatDto) -> Self {
        Self {
            title: dto.title,
            model_provider: dto.model_provider,
            model_id: dto.model_id,
            updated_at: Utc::now(),
        }
    }
}

