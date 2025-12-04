use anyhow::{Context, Result};
use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::db::{
    DbPool,
    models::{NewPreset, Preset, UpdatePreset},
    schema::presets,
};

#[async_trait]
pub trait TPresetRepository: Send + Sync {
    async fn create(&self, new_preset: NewPreset) -> Result<Preset>;
    async fn get(&self, id: Uuid, user_id: &str) -> Result<Option<Preset>>;
    async fn list(&self, user_id: &str) -> Result<Vec<Preset>>;
    async fn update(&self, id: Uuid, user_id: &str, update: UpdatePreset) -> Result<Preset>;
    async fn delete(&self, id: Uuid, user_id: &str) -> Result<bool>;
}

pub struct PresetRepository {
    pool: DbPool,
}

impl PresetRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TPresetRepository for PresetRepository {
    /// Create a new preset
    async fn create(&self, new_preset: NewPreset) -> Result<Preset> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        diesel::insert_into(presets::table)
            .values(&new_preset)
            .get_result(&mut conn)
            .await
            .context("Failed to create preset")
    }

    /// Get preset by ID
    async fn get(&self, id: Uuid, user_id: &str) -> Result<Option<Preset>> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        presets::table
            .filter(presets::id.eq(id))
            .filter(presets::user_id.eq(user_id))
            .first(&mut conn)
            .await
            .optional()
            .context("Failed to get preset")
    }

    /// List presets for a user
    async fn list(&self, user_id: &str) -> Result<Vec<Preset>> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        presets::table
            .filter(presets::user_id.eq(user_id))
            .order(presets::order_index.asc())
            .load(&mut conn)
            .await
            .context("Failed to list presets")
    }

    /// Update preset
    async fn update(&self, id: Uuid, user_id: &str, update: UpdatePreset) -> Result<Preset> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        diesel::update(presets::table)
            .filter(presets::id.eq(id))
            .filter(presets::user_id.eq(user_id))
            .set(&update)
            .get_result(&mut conn)
            .await
            .context("Failed to update preset")
    }

    /// Delete preset
    async fn delete(&self, id: Uuid, user_id: &str) -> Result<bool> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let deleted = diesel::delete(presets::table)
            .filter(presets::id.eq(id))
            .filter(presets::user_id.eq(user_id))
            .execute(&mut conn)
            .await
            .context("Failed to delete preset")?;

        Ok(deleted > 0)
    }
}
