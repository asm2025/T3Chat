use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::db::{
    DbPool,
    dto::{Pagination, ResultSet},
    models::{
        Chat, Message, NewChat, UpdateChat, 
        chat::{CreateChatDto, UpdateChatDto, NewMessage, UpdateMessage},
        message::{CreateMessageDto, UpdateMessageDto},
    },
    schema::{chats, messages},
};

#[async_trait]
pub trait TChatRepository: Send + Sync {
    // Chat methods
    async fn create(&self, dto: CreateChatDto) -> Result<Chat>;
    async fn get(&self, id: Uuid, user_id: &str) -> Result<Option<Chat>>;
    async fn get_by_chat_id(&self, chat_id: &str, user_id: &str) -> Result<Option<Chat>>;
    async fn list(&self, user_id: &str, pagination: Option<Pagination>) -> Result<ResultSet<Chat>>;
    async fn update(&self, id: Uuid, user_id: &str, dto: UpdateChatDto) -> Result<Chat>;
    async fn delete(&self, id: Uuid, user_id: &str) -> Result<()>;
    async fn archive(&self, id: Uuid, user_id: &str) -> Result<Chat>;
    async fn unarchive(&self, id: Uuid, user_id: &str) -> Result<Chat>;

    // Message methods
    async fn create_message(&self, dto: CreateMessageDto) -> Result<Message>;
    async fn get_message(&self, id: Uuid) -> Result<Option<Message>>;
    async fn list_messages(&self, chat_id: Uuid, user_id: &str) -> Result<Vec<Message>>;
    async fn update_message(&self, id: Uuid, chat_id: Uuid, user_id: &str, dto: UpdateMessageDto) -> Result<Message>;
    async fn delete_message(&self, id: Uuid, chat_id: Uuid, user_id: &str) -> Result<()>;
    async fn clear_messages(&self, chat_id: Uuid, user_id: &str) -> Result<()>;
    async fn get_next_sequence_number(&self, chat_id: Uuid) -> Result<i32>;
    async fn update_tokens_used(&self, id: Uuid, tokens: i32, model: &str) -> Result<()>;
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
    /// Create a new chat
    async fn create(&self, dto: CreateChatDto) -> Result<Chat> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let new_chat: NewChat = dto.into();
        diesel::insert_into(chats::table)
            .values(&new_chat)
            .get_result(&mut conn)
            .await
            .context("Failed to create chat")
    }

    /// Get chat by ID (supports both internal UUID and external chat_id)
    async fn get(&self, id: Uuid, user_id: &str) -> Result<Option<Chat>> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let id_str = id.to_string();
        chats::table
            .filter(chats::user_id.eq(user_id))
            .filter(chats::id.eq(id).or(chats::chat_id.eq(&id_str)))
            .filter(
                chats::is_archived
                    .eq(false)
                    .or(chats::is_archived.is_null()),
            )
            .first(&mut conn)
            .await
            .optional()
            .context("Failed to get chat")
    }

    /// Get chat by chat_id
    async fn get_by_chat_id(&self, chat_id: &str, user_id: &str) -> Result<Option<Chat>> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        chats::table
            .filter(chats::chat_id.eq(chat_id))
            .filter(chats::user_id.eq(user_id))
            .first(&mut conn)
            .await
            .optional()
            .context("Failed to get chat")
    }

    /// List chats for a user
    async fn list(&self, user_id: &str, pagination: Option<Pagination>) -> Result<ResultSet<Chat>> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        // Count total non-archived chats for this user
        let total = chats::table
            .filter(chats::user_id.eq(user_id))
            .filter(
                chats::is_archived
                    .eq(false)
                    .or(chats::is_archived.is_null()),
            )
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .context("Failed to count chats")? as u64;

        if total == 0 {
            return Ok(ResultSet {
                data: vec![],
                total: 0,
                pagination,
            });
        }

        // Build query
        let mut query = chats::table
            .filter(chats::user_id.eq(user_id))
            .filter(
                chats::is_archived
                    .eq(false)
                    .or(chats::is_archived.is_null()),
            )
            .order(chats::updated_at.desc())
            .into_boxed();

        if let Some(p) = pagination {
            query = query
                .offset(((p.page - 1) * p.page_size) as i64)
                .limit(p.page_size as i64);
        }

        let data = query
            .load::<Chat>(&mut conn)
            .await
            .context("Failed to list chats")?;

        Ok(ResultSet {
            data,
            total,
            pagination,
        })
    }

    /// Update chat
    async fn update(&self, id: Uuid, user_id: &str, dto: UpdateChatDto) -> Result<Chat> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        // Check if chat exists and belongs to user
        let _existing = chats::table
            .filter(chats::id.eq(id))
            .filter(chats::user_id.eq(user_id))
            .filter(
                chats::is_archived
                    .eq(false)
                    .or(chats::is_archived.is_null()),
            )
            .first::<Chat>(&mut conn)
            .await
            .optional()
            .context("Failed to check chat")?
            .ok_or_else(|| anyhow::anyhow!("Chat not found"))?;

        let update: UpdateChat = dto.into();
        diesel::update(chats::table)
            .filter(chats::id.eq(id))
            .filter(chats::user_id.eq(user_id))
            .set(&update)
            .get_result(&mut conn)
            .await
            .context("Failed to update chat")
    }

    /// Delete chat (soft delete by archiving)
    async fn delete(&self, id: Uuid, user_id: &str) -> Result<()> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        // Check if chat exists and belongs to user
        let _existing = chats::table
            .filter(chats::id.eq(id))
            .filter(chats::user_id.eq(user_id))
            .filter(
                chats::is_archived
                    .eq(false)
                    .or(chats::is_archived.is_null()),
            )
            .first::<Chat>(&mut conn)
            .await
            .optional()
            .context("Failed to check chat")?
            .ok_or_else(|| anyhow::anyhow!("Chat not found"))?;

        // Soft delete by setting is_archived
        diesel::update(chats::table)
            .filter(chats::id.eq(id))
            .filter(chats::user_id.eq(user_id))
            .set(chats::is_archived.eq(Some(true)))
            .execute(&mut conn)
            .await
            .context("Failed to delete chat")?;

        Ok(())
    }

    /// Archive chat
    async fn archive(&self, id: Uuid, user_id: &str) -> Result<Chat> {
        let dto = UpdateChatDto {
            title: None,
            model_provider: None,
            model_id: None,
        };
        let mut update: UpdateChat = dto.into();
        update.is_archived = Some(true);
        update.updated_at = Utc::now();
        
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        diesel::update(chats::table)
            .filter(chats::id.eq(id))
            .filter(chats::user_id.eq(user_id))
            .set(&update)
            .get_result(&mut conn)
            .await
            .context("Failed to archive chat")
    }

    /// Unarchive chat
    async fn unarchive(&self, id: Uuid, user_id: &str) -> Result<Chat> {
        let dto = UpdateChatDto {
            title: None,
            model_provider: None,
            model_id: None,
        };
        let mut update: UpdateChat = dto.into();
        update.is_archived = Some(false);
        update.updated_at = Utc::now();
        
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        diesel::update(chats::table)
            .filter(chats::id.eq(id))
            .filter(chats::user_id.eq(user_id))
            .set(&update)
            .get_result(&mut conn)
            .await
            .context("Failed to unarchive chat")
    }

    // ==========================================
    // MESSAGE OPERATIONS
    // ==========================================

    /// Create a new message
    async fn create_message(&self, dto: CreateMessageDto) -> Result<Message> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let new_message: NewMessage = dto.into();
        diesel::insert_into(messages::table)
            .values(&new_message)
            .get_result(&mut conn)
            .await
            .context("Failed to create message")
    }

    /// Get message by ID
    async fn get_message(&self, id: Uuid) -> Result<Option<Message>> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        messages::table
            .filter(messages::id.eq(id))
            .first(&mut conn)
            .await
            .optional()
            .context("Failed to get message")
    }

    /// List messages for a chat
    async fn list_messages(&self, chat_id: Uuid, _user_id: &str) -> Result<Vec<Message>> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        messages::table
            .filter(messages::chat_id.eq(chat_id))
            .order(messages::created_at.asc())
            .load(&mut conn)
            .await
            .context("Failed to list messages")
    }

    /// Update message
    async fn update_message(&self, id: Uuid, chat_id: Uuid, user_id: &str, dto: UpdateMessageDto) -> Result<Message> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        // Verify chat belongs to user
        let _chat = chats::table
            .filter(chats::id.eq(chat_id))
            .filter(chats::user_id.eq(user_id))
            .first::<Chat>(&mut conn)
            .await
            .optional()
            .context("Failed to verify chat")?
            .ok_or_else(|| anyhow::anyhow!("Chat not found"))?;

        // Check if message exists and belongs to the chat
        let _existing = messages::table
            .filter(messages::id.eq(id))
            .filter(messages::chat_id.eq(chat_id))
            .first::<Message>(&mut conn)
            .await
            .optional()
            .context("Failed to check message")?
            .ok_or_else(|| anyhow::anyhow!("Message not found"))?;

        let update: UpdateMessage = dto.into();
        diesel::update(messages::table)
            .filter(messages::id.eq(id))
            .filter(messages::chat_id.eq(chat_id))
            .set(&update)
            .get_result(&mut conn)
            .await
            .context("Failed to update message")
    }

    /// Delete message
    async fn delete_message(&self, id: Uuid, chat_id: Uuid, user_id: &str) -> Result<()> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        // Verify chat belongs to user
        let _chat = chats::table
            .filter(chats::id.eq(chat_id))
            .filter(chats::user_id.eq(user_id))
            .first::<Chat>(&mut conn)
            .await
            .optional()
            .context("Failed to verify chat")?
            .ok_or_else(|| anyhow::anyhow!("Chat not found"))?;

        // Check if message exists and belongs to the chat
        let _existing = messages::table
            .filter(messages::id.eq(id))
            .filter(messages::chat_id.eq(chat_id))
            .first::<Message>(&mut conn)
            .await
            .optional()
            .context("Failed to check message")?
            .ok_or_else(|| anyhow::anyhow!("Message not found"))?;

        diesel::delete(messages::table)
            .filter(messages::id.eq(id))
            .execute(&mut conn)
            .await
            .context("Failed to delete message")?;

        Ok(())
    }

    /// Clear all messages in a chat
    async fn clear_messages(&self, chat_id: Uuid, user_id: &str) -> Result<()> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        // Verify chat belongs to user
        let _chat = chats::table
            .filter(chats::id.eq(chat_id))
            .filter(chats::user_id.eq(user_id))
            .first::<Chat>(&mut conn)
            .await
            .optional()
            .context("Failed to verify chat")?
            .ok_or_else(|| anyhow::anyhow!("Chat not found"))?;

        diesel::delete(messages::table)
            .filter(messages::chat_id.eq(chat_id))
            .execute(&mut conn)
            .await
            .context("Failed to clear messages")?;

        Ok(())
    }

    /// Get next sequence number for a chat (counts existing messages + 1)
    async fn get_next_sequence_number(&self, chat_id: Uuid) -> Result<i32> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let count: i64 = messages::table
            .filter(messages::chat_id.eq(chat_id))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .context("Failed to count messages")?;

        Ok((count + 1) as i32)
    }

    /// Update token count for a message
    async fn update_tokens_used(&self, id: Uuid, tokens: i32, _model: &str) -> Result<()> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        // Check if message exists
        let _existing = messages::table
            .find(id)
            .first::<Message>(&mut conn)
            .await
            .optional()
            .context("Failed to check message")?
            .ok_or_else(|| anyhow::anyhow!("Message not found"))?;

        let update = UpdateMessage {
            text: None,
            content: None,
            token_count: Some(tokens),
            finish_reason: None,
            error: None,
            updated_at: Utc::now(),
        };

        diesel::update(messages::table.find(id))
            .set(&update)
            .execute(&mut conn)
            .await
            .context("Failed to update tokens")?;

        Ok(())
    }
}
