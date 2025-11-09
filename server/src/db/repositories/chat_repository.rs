use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use emixdiesel::{Error, Result};
use uuid::Uuid;

use crate::db::dto::{Pagination, ResultSet};
use crate::db::models::{ChatModel, CreateChatDto, NewChat, UpdateChat, UpdateChatDto};
use crate::db::{DbPool, schema::chats};

#[async_trait]
pub trait IChatRepository: Send + Sync {
    async fn list_by_user(
        &self,
        user_id: &str,
        pagination: Option<Pagination>,
    ) -> Result<ResultSet<ChatModel>>;
    async fn get_by_id(&self, id: Uuid, user_id: &str) -> Result<Option<ChatModel>>;
    async fn create(&self, model: CreateChatDto) -> Result<ChatModel>;
    async fn update(&self, id: Uuid, user_id: &str, model: UpdateChatDto) -> Result<ChatModel>;
    async fn delete(&self, id: Uuid, user_id: &str) -> Result<()>;
}

pub struct ChatRepository {
    pool: DbPool,
}

impl ChatRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl IChatRepository for ChatRepository {
    async fn list_by_user(
        &self,
        user_id: &str,
        pagination: Option<Pagination>,
    ) -> Result<ResultSet<ChatModel>> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        // Count total non-deleted chats for this user
        let total = chats::table
            .filter(chats::user_id.eq(user_id))
            .filter(chats::deleted_at.is_null())
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(Error::from_std_error)? as u64;

        // Build query
        let mut query = chats::table
            .filter(chats::user_id.eq(user_id))
            .filter(chats::deleted_at.is_null())
            .order(chats::updated_at.desc())
            .into_boxed();

        if let Some(p) = pagination {
            query = query
                .offset(((p.page - 1) * p.page_size) as i64)
                .limit(p.page_size as i64);
        }

        let data = query
            .load::<ChatModel>(&mut conn)
            .await
            .map_err(Error::from_std_error)?;

        Ok(ResultSet {
            data,
            total,
            pagination,
        })
    }

    async fn get_by_id(&self, id: Uuid, user_id: &str) -> Result<Option<ChatModel>> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        chats::table
            .filter(chats::id.eq(id))
            .filter(chats::user_id.eq(user_id))
            .filter(chats::deleted_at.is_null())
            .first::<ChatModel>(&mut conn)
            .await
            .optional()
            .map_err(Error::from_std_error)
    }

    async fn create(&self, model: CreateChatDto) -> Result<ChatModel> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        let new_chat: NewChat = model.into();

        diesel::insert_into(chats::table)
            .values(&new_chat)
            .get_result(&mut conn)
            .await
            .map_err(Error::from_std_error)
    }

    async fn update(&self, id: Uuid, user_id: &str, model: UpdateChatDto) -> Result<ChatModel> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        // Check if chat exists and belongs to user
        let _existing = chats::table
            .filter(chats::id.eq(id))
            .filter(chats::user_id.eq(user_id))
            .filter(chats::deleted_at.is_null())
            .first::<ChatModel>(&mut conn)
            .await
            .optional()
            .map_err(Error::from_std_error)?
            .ok_or_else(|| Error::from_other_error("Chat not found".to_string()))?;

        let update_chat: UpdateChat = model.into();

        diesel::update(chats::table.filter(chats::id.eq(id)))
            .set(&update_chat)
            .get_result(&mut conn)
            .await
            .map_err(Error::from_std_error)
    }

    async fn delete(&self, id: Uuid, user_id: &str) -> Result<()> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        // Check if chat exists and belongs to user
        let _existing = chats::table
            .filter(chats::id.eq(id))
            .filter(chats::user_id.eq(user_id))
            .filter(chats::deleted_at.is_null())
            .first::<ChatModel>(&mut conn)
            .await
            .optional()
            .map_err(Error::from_std_error)?
            .ok_or_else(|| Error::from_other_error("Chat not found".to_string()))?;

        // Soft delete by setting deleted_at
        diesel::update(chats::table.filter(chats::id.eq(id)))
            .set(chats::deleted_at.eq(Some(chrono::Utc::now())))
            .execute(&mut conn)
            .await
            .map_err(Error::from_std_error)?;

        Ok(())
    }
}
