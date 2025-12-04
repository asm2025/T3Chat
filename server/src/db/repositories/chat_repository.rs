use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use emixdiesel::{Error, Result};
use uuid::Uuid;

use crate::db::dto::{Pagination, ResultSet};
use crate::db::models::{
    ChatModel,
    // Use conversation models from the actual schema
    Conversation,
    CreateChatDto,
    CreateMessageDto,
    Message,
    NewConversation,
    UpdateChatDto,
    UpdateConversation,
    UpdateMessageDto,
    conversation::NewMessage, // Explicitly use conversation types
    conversation::UpdateMessage,
};

// Aliases for legacy API compatibility
type UpdateChat = UpdateConversation;
use crate::db::{
    DbPool,
    schema::{conversations as chats, messages}, // Map conversations to chats for legacy compatibility
};

// Type alias for API compatibility - the repository now returns conversation Message model
type MessageModel = Message;

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

        // Count total non-archived conversations for this user
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
            .map_err(Error::from_std_error)? as u64;

        if total == 0 {
            return Ok(ResultSet {
                data: vec![],
                total: 0,
                pagination,
            });
        }

        // Build query - returns Conversation, convert to ChatModel
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

        let conversations = query
            .load::<Conversation>(&mut conn)
            .await
            .map_err(Error::from_std_error)?;

        // Convert Conversation to ChatModel for API compatibility
        let data = conversations.into_iter().map(ChatModel::from).collect();

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

        let conversation = chats::table
            .filter(chats::id.eq(id))
            .filter(chats::user_id.eq(user_id))
            .filter(
                chats::is_archived
                    .eq(false)
                    .or(chats::is_archived.is_null()),
            )
            .first::<Conversation>(&mut conn)
            .await
            .optional()
            .map_err(Error::from_std_error)?;

        Ok(conversation.map(ChatModel::from))
    }

    async fn create(&self, model: CreateChatDto) -> Result<ChatModel> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        // Convert CreateChatDto to NewConversation
        let new_conversation = NewConversation {
            id: Some(Uuid::new_v4()),
            conversation_id: Uuid::new_v4().to_string(),
            user_id: model.user_id,
            title: Some(model.title),
            endpoint: model.model_provider.as_str().to_string(),
            model: model.model_id,
            model_label: None,
            model_parameters: None,
            system_message: None,
            instructions: None,
            feature_flags: None,
            agent_id: None,
            assistant_id: None,
            agent_options: None,
        };

        let conversation = diesel::insert_into(chats::table)
            .values(&new_conversation)
            .get_result::<Conversation>(&mut conn)
            .await
            .map_err(Error::from_std_error)?;

        Ok(ChatModel::from(conversation))
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
            .filter(
                chats::is_archived
                    .eq(false)
                    .or(chats::is_archived.is_null()),
            )
            .first::<Conversation>(&mut conn)
            .await
            .optional()
            .map_err(Error::from_std_error)?
            .ok_or_else(|| Error::NotFound("Chat not found".to_string()))?;

        // Convert UpdateChatDto to UpdateConversation
        let update_conversation = UpdateConversation {
            title: model.title,
            endpoint: model.model_provider.map(|p| p.as_str().to_string()),
            model: model.model_id,
            model_label: None,
            model_parameters: None,
            system_message: None,
            instructions: None,
            feature_flags: None,
            agent_id: None,
            assistant_id: None,
            agent_options: None,
            is_archived: None,
            updated_at: chrono::Utc::now(),
        };

        let conversation = diesel::update(chats::table.filter(chats::id.eq(id)))
            .set(&update_conversation)
            .get_result::<Conversation>(&mut conn)
            .await
            .map_err(Error::from_std_error)?;

        Ok(ChatModel::from(conversation))
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
            .filter(
                chats::is_archived
                    .eq(false)
                    .or(chats::is_archived.is_null()),
            )
            .first::<Conversation>(&mut conn)
            .await
            .optional()
            .map_err(Error::from_std_error)?
            .ok_or_else(|| Error::NotFound("Chat not found".to_string()))?;

        // Soft delete by setting is_archived
        diesel::update(chats::table.filter(chats::id.eq(id)))
            .set(chats::is_archived.eq(Some(true)))
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
            .first::<Conversation>(&mut conn)
            .await
            .optional()
            .map_err(Error::from_std_error)?
            .ok_or_else(|| Error::NotFound("Chat not found".to_string()))?;

        // Query messages from conversation - returns Message (conversation model), which is aliased as MessageModel
        messages::table
            .filter(messages::conversation_id.eq(chat_id))
            .order(messages::created_at.asc())
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

        // Note: messages table doesn't have sequence_number in the new schema
        // We'll just count existing messages + 1 for legacy API compatibility
        let count: i64 = messages::table
            .filter(messages::conversation_id.eq(chat_id))
            .count()
            .first::<i64>(&mut conn)
            .await
            .map_err(Error::from_std_error)?;

        Ok((count + 1) as i32)
    }

    async fn update_tokens_used(&self, id: Uuid, tokens: i32, _model: &str) -> Result<()> {
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
            .ok_or_else(|| Error::NotFound("Message not found".to_string()))?;

        let update = UpdateMessage {
            text: None,
            content: None,
            token_count: Some(tokens),
            finish_reason: None,
            error: None,
            updated_at: chrono::Utc::now(),
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
            .filter(
                chats::is_archived
                    .eq(false)
                    .or(chats::is_archived.is_null()),
            )
            .first::<Conversation>(&mut conn)
            .await
            .optional()
            .map_err(Error::from_std_error)?
            .ok_or_else(|| Error::NotFound("Chat not found".to_string()))?;

        // Check if message exists and belongs to the chat
        let _existing = messages::table
            .filter(messages::id.eq(id))
            .filter(messages::conversation_id.eq(chat_id))
            .first::<Message>(&mut conn)
            .await
            .optional()
            .map_err(Error::from_std_error)?
            .ok_or_else(|| Error::NotFound("Message not found".to_string()))?;

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
            .filter(
                chats::is_archived
                    .eq(false)
                    .or(chats::is_archived.is_null()),
            )
            .first::<Conversation>(&mut conn)
            .await
            .optional()
            .map_err(Error::from_std_error)?
            .ok_or_else(|| Error::NotFound("Chat not found".to_string()))?;

        // Check if message exists and belongs to the chat
        let _existing = messages::table
            .filter(messages::id.eq(id))
            .filter(messages::conversation_id.eq(chat_id))
            .first::<Message>(&mut conn)
            .await
            .optional()
            .map_err(Error::from_std_error)?
            .ok_or_else(|| Error::NotFound("Message not found".to_string()))?;

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
            .filter(
                chats::is_archived
                    .eq(false)
                    .or(chats::is_archived.is_null()),
            )
            .first::<Conversation>(&mut conn)
            .await
            .optional()
            .map_err(Error::from_std_error)?
            .ok_or_else(|| Error::NotFound("Chat not found".to_string()))?;

        diesel::delete(messages::table.filter(messages::conversation_id.eq(chat_id)))
            .execute(&mut conn)
            .await
            .map_err(Error::from_std_error)?;

        Ok(())
    }
}
