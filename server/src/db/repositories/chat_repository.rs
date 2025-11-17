use async_trait::async_trait;
use diesel::dsl::max;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use emixdiesel::{Error, Result};
use uuid::Uuid;

use crate::db::dto::{Pagination, ResultSet};
use crate::db::models::{
    ChatModel, CreateChatDto, CreateMessageDto, MessageModel, NewChat, NewMessage, UpdateChat,
    UpdateChatDto, UpdateMessage, UpdateMessageDto,
};
use crate::db::{
    DbPool,
    schema::{chats, messages},
};

#[async_trait]
pub trait TChatRepository: Send + Sync {
    // Chat methods
    async fn list(
        &self,
        user_id: &str,
        pagination: Option<Pagination>,
    ) -> Result<ResultSet<ChatModel>>;
    async fn get(&self, id: Uuid, user_id: &str) -> Result<Option<ChatModel>>;
    async fn create(&self, model: CreateChatDto) -> Result<ChatModel>;
    async fn update(&self, id: Uuid, user_id: &str, model: UpdateChatDto) -> Result<ChatModel>;
    async fn delete(&self, id: Uuid, user_id: &str) -> Result<()>;

    // Message methods
    async fn list_messages(&self, chat_id: Uuid, user_id: &str) -> Result<Vec<MessageModel>>;
    async fn create_message(&self, model: CreateMessageDto) -> Result<MessageModel>;
    async fn get_next_sequence_number(&self, chat_id: Uuid) -> Result<i32>;
    async fn update_tokens_used(&self, id: Uuid, tokens: i32, model: &str) -> Result<()>;
    async fn update_message(
        &self,
        id: Uuid,
        chat_id: Uuid,
        user_id: &str,
        model: UpdateMessageDto,
    ) -> Result<MessageModel>;
    async fn delete_message(&self, id: Uuid, chat_id: Uuid, user_id: &str) -> Result<()>;
    async fn clear_messages(&self, chat_id: Uuid, user_id: &str) -> Result<()>;
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
impl TChatRepository for ChatRepository {
    async fn list(
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

    async fn get(&self, id: Uuid, user_id: &str) -> Result<Option<ChatModel>> {
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

    async fn list_messages(&self, chat_id: Uuid, user_id: &str) -> Result<Vec<MessageModel>> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        // Verify chat belongs to user
        let _chat = chats::table
            .filter(chats::id.eq(chat_id))
            .filter(chats::user_id.eq(user_id))
            .first::<ChatModel>(&mut conn)
            .await
            .optional()
            .map_err(Error::from_std_error)?
            .ok_or_else(|| Error::from_other_error("Chat not found".to_string()))?;

        messages::table
            .filter(messages::chat_id.eq(chat_id))
            .order(messages::sequence_number.asc())
            .load::<MessageModel>(&mut conn)
            .await
            .map_err(Error::from_std_error)
    }

    async fn create_message(&self, model: CreateMessageDto) -> Result<MessageModel> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        let new_message: NewMessage = model.into();

        diesel::insert_into(messages::table)
            .values(&new_message)
            .get_result(&mut conn)
            .await
            .map_err(Error::from_std_error)
    }

    async fn get_next_sequence_number(&self, chat_id: Uuid) -> Result<i32> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        let max_seq: Option<i32> = messages::table
            .filter(messages::chat_id.eq(chat_id))
            .select(max(messages::sequence_number))
            .first::<Option<i32>>(&mut conn)
            .await
            .map_err(Error::from_std_error)?;

        Ok(max_seq.unwrap_or(0) + 1)
    }

    async fn update_tokens_used(&self, id: Uuid, tokens: i32, model: &str) -> Result<()> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        // Check if message exists
        let _existing = messages::table
            .find(id)
            .first::<MessageModel>(&mut conn)
            .await
            .optional()
            .map_err(Error::from_std_error)?
            .ok_or_else(|| Error::from_other_error("Message not found".to_string()))?;

        let update = UpdateMessage {
            content: None,
            metadata: None,
            tokens_used: Some(tokens),
            model_used: Some(model.to_string()),
        };

        diesel::update(messages::table.find(id))
            .set(&update)
            .execute(&mut conn)
            .await
            .map_err(Error::from_std_error)?;

        Ok(())
    }

    async fn update_message(
        &self,
        id: Uuid,
        chat_id: Uuid,
        user_id: &str,
        model: UpdateMessageDto,
    ) -> Result<MessageModel> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        // Verify chat belongs to user
        let _chat = chats::table
            .filter(chats::id.eq(chat_id))
            .filter(chats::user_id.eq(user_id))
            .filter(chats::deleted_at.is_null())
            .first::<ChatModel>(&mut conn)
            .await
            .optional()
            .map_err(Error::from_std_error)?
            .ok_or_else(|| Error::from_other_error("Chat not found".to_string()))?;

        // Check if message exists and belongs to the chat
        let _existing = messages::table
            .filter(messages::id.eq(id))
            .filter(messages::chat_id.eq(chat_id))
            .first::<MessageModel>(&mut conn)
            .await
            .optional()
            .map_err(Error::from_std_error)?
            .ok_or_else(|| Error::from_other_error("Message not found".to_string()))?;

        let update: UpdateMessage = model.into();

        diesel::update(messages::table.filter(messages::id.eq(id)))
            .set(&update)
            .get_result(&mut conn)
            .await
            .map_err(Error::from_std_error)
    }

    async fn delete_message(&self, id: Uuid, chat_id: Uuid, user_id: &str) -> Result<()> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        // Verify chat belongs to user
        let _chat = chats::table
            .filter(chats::id.eq(chat_id))
            .filter(chats::user_id.eq(user_id))
            .filter(chats::deleted_at.is_null())
            .first::<ChatModel>(&mut conn)
            .await
            .optional()
            .map_err(Error::from_std_error)?
            .ok_or_else(|| Error::from_other_error("Chat not found".to_string()))?;

        // Check if message exists and belongs to the chat
        let _existing = messages::table
            .filter(messages::id.eq(id))
            .filter(messages::chat_id.eq(chat_id))
            .first::<MessageModel>(&mut conn)
            .await
            .optional()
            .map_err(Error::from_std_error)?
            .ok_or_else(|| Error::from_other_error("Message not found".to_string()))?;

        diesel::delete(messages::table.filter(messages::id.eq(id)))
            .execute(&mut conn)
            .await
            .map_err(Error::from_std_error)?;

        Ok(())
    }

    async fn clear_messages(&self, chat_id: Uuid, user_id: &str) -> Result<()> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        // Verify chat belongs to user
        let _chat = chats::table
            .filter(chats::id.eq(chat_id))
            .filter(chats::user_id.eq(user_id))
            .filter(chats::deleted_at.is_null())
            .first::<ChatModel>(&mut conn)
            .await
            .optional()
            .map_err(Error::from_std_error)?
            .ok_or_else(|| Error::from_other_error("Chat not found".to_string()))?;

        diesel::delete(messages::table.filter(messages::chat_id.eq(chat_id)))
            .execute(&mut conn)
            .await
            .map_err(Error::from_std_error)?;

        Ok(())
    }
}
