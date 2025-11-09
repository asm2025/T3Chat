use async_trait::async_trait;
use diesel::dsl::max;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use emixdiesel::{Error, Result};
use uuid::Uuid;

use crate::db::models::{CreateMessageDto, MessageModel, NewMessage, UpdateMessage};
use crate::db::{
    DbPool,
    schema::{chats, messages},
};

#[async_trait]
pub trait IMessageRepository: Send + Sync {
    async fn list_by_chat(&self, chat_id: Uuid, user_id: &str) -> Result<Vec<MessageModel>>;
    async fn create(&self, model: CreateMessageDto) -> Result<MessageModel>;
    async fn get_next_sequence_number(&self, chat_id: Uuid) -> Result<i32>;
    async fn update_tokens_used(&self, id: Uuid, tokens: i32, model: &str) -> Result<()>;
}

pub struct MessageRepository {
    pool: DbPool,
}

impl MessageRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl IMessageRepository for MessageRepository {
    async fn list_by_chat(&self, chat_id: Uuid, user_id: &str) -> Result<Vec<MessageModel>> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        // Verify chat belongs to user
        let _chat = chats::table
            .filter(chats::id.eq(chat_id))
            .filter(chats::user_id.eq(user_id))
            .first::<crate::db::models::ChatModel>(&mut conn)
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

    async fn create(&self, model: CreateMessageDto) -> Result<MessageModel> {
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
}
