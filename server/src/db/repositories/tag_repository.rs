use anyhow::{Context, Result};
use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::db::{
    DbPool,
    models::{NewChatTag, NewTag, Tag, UpdateTag},
    schema::{chat_tags_map, tags},
};

#[async_trait]
pub trait TTagRepository: Send + Sync {
    async fn create(&self, new_tag: NewTag) -> Result<Tag>;
    async fn get(&self, id: Uuid, user_id: &str) -> Result<Option<Tag>>;
    async fn list(&self, user_id: &str) -> Result<Vec<Tag>>;
    async fn update(&self, id: Uuid, user_id: &str, update: UpdateTag) -> Result<Tag>;
    async fn delete(&self, id: Uuid, user_id: &str) -> Result<bool>;
    async fn add_to_chat(&self, chat_id: Uuid, tag_id: Uuid) -> Result<()>;
    async fn remove_from_chat(&self, chat_id: Uuid, tag_id: Uuid) -> Result<bool>;
}

pub struct TagRepository {
    pool: DbPool,
}

impl TagRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TTagRepository for TagRepository {
    /// Create a new tag
    async fn create(&self, new_tag: NewTag) -> Result<Tag> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        diesel::insert_into(tags::table)
            .values(&new_tag)
            .get_result(&mut conn)
            .await
            .context("Failed to create tag")
    }

    /// Get tag by ID
    async fn get(&self, id: Uuid, user_id: &str) -> Result<Option<Tag>> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        tags::table
            .filter(tags::id.eq(id))
            .filter(tags::user_id.eq(user_id))
            .first(&mut conn)
            .await
            .optional()
            .context("Failed to get tag")
    }

    /// List tags for a user
    async fn list(&self, user_id: &str) -> Result<Vec<Tag>> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        tags::table
            .filter(tags::user_id.eq(user_id))
            .order(tags::position.asc())
            .load(&mut conn)
            .await
            .context("Failed to list tags")
    }

    /// Update tag
    async fn update(&self, id: Uuid, user_id: &str, update: UpdateTag) -> Result<Tag> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        diesel::update(tags::table)
            .filter(tags::id.eq(id))
            .filter(tags::user_id.eq(user_id))
            .set(&update)
            .get_result(&mut conn)
            .await
            .context("Failed to update tag")
    }

    /// Delete tag
    async fn delete(&self, id: Uuid, user_id: &str) -> Result<bool> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let deleted = diesel::delete(tags::table)
            .filter(tags::id.eq(id))
            .filter(tags::user_id.eq(user_id))
            .execute(&mut conn)
            .await
            .context("Failed to delete tag")?;

        Ok(deleted > 0)
    }

    /// Add tag to chat
    async fn add_to_chat(&self, chat_id: Uuid, tag_id: Uuid) -> Result<()> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let new_mapping = NewChatTag { chat_id, tag_id };

        diesel::insert_into(chat_tags_map::table)
            .values(&new_mapping)
            .execute(&mut conn)
            .await
            .context("Failed to add tag to chat")?;

        Ok(())
    }

    /// Remove tag from chat
    async fn remove_from_chat(&self, chat_id: Uuid, tag_id: Uuid) -> Result<bool> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let deleted = diesel::delete(chat_tags_map::table)
            .filter(chat_tags_map::chat_id.eq(chat_id))
            .filter(chat_tags_map::tag_id.eq(tag_id))
            .execute(&mut conn)
            .await
            .context("Failed to remove tag from chat")?;

        Ok(deleted > 0)
    }
}
