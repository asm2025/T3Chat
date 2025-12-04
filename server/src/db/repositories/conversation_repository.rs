use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::db::{
    DbPool,
    models::{
        Conversation, Message, NewConversation, UpdateConversation, conversation::NewMessage,
        conversation::UpdateMessage,
    },
    schema::{conversations, messages},
};

#[async_trait]
pub trait TConversationRepository: Send + Sync {
    // Conversation methods
    async fn create(&self, new_conversation: NewConversation) -> Result<Conversation>;
    async fn get(&self, id: Uuid, user_id: &str) -> Result<Option<Conversation>>;
    async fn get_by_conversation_id(&self, conversation_id: &str, user_id: &str) -> Result<Option<Conversation>>;
    async fn list(&self, user_id: &str, include_archived: bool) -> Result<Vec<Conversation>>;
    async fn update(&self, id: Uuid, user_id: &str, update: UpdateConversation) -> Result<Conversation>;
    async fn delete(&self, id: Uuid, user_id: &str) -> Result<bool>;
    async fn archive(&self, id: Uuid, user_id: &str) -> Result<Conversation>;
    async fn unarchive(&self, id: Uuid, user_id: &str) -> Result<Conversation>;

    // Message methods
    async fn create_message(&self, new_message: NewMessage) -> Result<Message>;
    async fn get_message(&self, id: Uuid) -> Result<Option<Message>>;
    async fn list_messages(&self, conversation_id: Uuid) -> Result<Vec<Message>>;
    async fn update_message(&self, id: Uuid, update: UpdateMessage) -> Result<Message>;
    async fn delete_message(&self, id: Uuid) -> Result<bool>;
    async fn delete_all_messages(&self, conversation_id: Uuid) -> Result<usize>;
}

pub struct ConversationRepository {
    pool: DbPool,
}

impl ConversationRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TConversationRepository for ConversationRepository {
    /// Create a new conversation
    async fn create(&self, new_conversation: NewConversation) -> Result<Conversation> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        diesel::insert_into(conversations::table)
            .values(&new_conversation)
            .get_result(&mut conn)
            .await
            .context("Failed to create conversation")
    }

    /// Get conversation by ID
    async fn get(&self, id: Uuid, user_id: &str) -> Result<Option<Conversation>> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        conversations::table
            .filter(conversations::id.eq(id))
            .filter(conversations::user_id.eq(user_id))
            .first(&mut conn)
            .await
            .optional()
            .context("Failed to get conversation")
    }

    /// Get conversation by conversation_id
    async fn get_by_conversation_id(
        &self,
        conversation_id: &str,
        user_id: &str,
    ) -> Result<Option<Conversation>> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        conversations::table
            .filter(conversations::conversation_id.eq(conversation_id))
            .filter(conversations::user_id.eq(user_id))
            .first(&mut conn)
            .await
            .optional()
            .context("Failed to get conversation")
    }

    /// List conversations for a user
    async fn list(
        &self,
        user_id: &str,
        include_archived: bool,
    ) -> Result<Vec<Conversation>> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let mut query = conversations::table
            .filter(conversations::user_id.eq(user_id))
            .into_boxed();

        if !include_archived {
            query = query.filter(
                conversations::is_archived
                    .eq(false)
                    .or(conversations::is_archived.is_null()),
            );
        }

        query
            .order(conversations::updated_at.desc())
            .load(&mut conn)
            .await
            .context("Failed to list conversations")
    }

    /// Update conversation
    async fn update(
        &self,
        id: Uuid,
        user_id: &str,
        update: UpdateConversation,
    ) -> Result<Conversation> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        diesel::update(conversations::table)
            .filter(conversations::id.eq(id))
            .filter(conversations::user_id.eq(user_id))
            .set(&update)
            .get_result(&mut conn)
            .await
            .context("Failed to update conversation")
    }

    /// Delete conversation
    async fn delete(&self, id: Uuid, user_id: &str) -> Result<bool> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let deleted = diesel::delete(conversations::table)
            .filter(conversations::id.eq(id))
            .filter(conversations::user_id.eq(user_id))
            .execute(&mut conn)
            .await
            .context("Failed to delete conversation")?;

        Ok(deleted > 0)
    }

    /// Archive conversation
    async fn archive(&self, id: Uuid, user_id: &str) -> Result<Conversation> {
        let update = UpdateConversation {
            is_archived: Some(true),
            updated_at: Utc::now(),
            ..Default::default()
        };
        self.update(id, user_id, update).await
    }

    /// Unarchive conversation
    async fn unarchive(&self, id: Uuid, user_id: &str) -> Result<Conversation> {
        let update = UpdateConversation {
            is_archived: Some(false),
            updated_at: Utc::now(),
            ..Default::default()
        };
        self.update(id, user_id, update).await
    }

    // ==========================================
    // MESSAGE OPERATIONS
    // ==========================================

    /// Create a new message
    async fn create_message(&self, new_message: NewMessage) -> Result<Message> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

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

    /// List messages for a conversation
    async fn list_messages(&self, conversation_id: Uuid) -> Result<Vec<Message>> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        messages::table
            .filter(messages::conversation_id.eq(conversation_id))
            .order(messages::created_at.asc())
            .load(&mut conn)
            .await
            .context("Failed to list messages")
    }

    /// Update message
    async fn update_message(&self, id: Uuid, update: UpdateMessage) -> Result<Message> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        diesel::update(messages::table)
            .filter(messages::id.eq(id))
            .set(&update)
            .get_result(&mut conn)
            .await
            .context("Failed to update message")
    }

    /// Delete message
    async fn delete_message(&self, id: Uuid) -> Result<bool> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let deleted = diesel::delete(messages::table)
            .filter(messages::id.eq(id))
            .execute(&mut conn)
            .await
            .context("Failed to delete message")?;

        Ok(deleted > 0)
    }

    /// Delete all messages in a conversation
    async fn delete_all_messages(&self, conversation_id: Uuid) -> Result<usize> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        diesel::delete(messages::table)
            .filter(messages::conversation_id.eq(conversation_id))
            .execute(&mut conn)
            .await
            .context("Failed to delete messages")
    }
}

impl Default for UpdateConversation {
    fn default() -> Self {
        Self {
            title: None,
            endpoint: None,
            model: None,
            model_label: None,
            model_parameters: None,
            system_message: None,
            instructions: None,
            feature_flags: None,
            agent_id: None,
            assistant_id: None,
            agent_options: None,
            is_archived: None,
            updated_at: Utc::now(),
        }
    }
}
