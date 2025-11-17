use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use emixdiesel::{Error, Result};
use uuid::Uuid;

use crate::db::models::{AiModelModel, AiProvider};
use crate::db::{DbPool, schema::ai_models};

#[async_trait]
pub trait TAiModelRepository: Send + Sync {
    async fn list_active(&self) -> Result<Vec<AiModelModel>>;
    async fn list(&self) -> Result<Vec<AiModelModel>>;
    async fn get_by_provider_and_model_id(
        &self,
        provider: &AiProvider,
        model_id: &str,
    ) -> Result<Option<AiModelModel>>;
    async fn get(&self, id: Uuid) -> Result<Option<AiModelModel>>;
    async fn enable_for_user(&self, user_id: &str, model_id: Uuid) -> Result<()>;
    async fn disable_for_user(&self, user_id: &str, model_id: Uuid) -> Result<()>;
    async fn list_for_user(&self, user_id: &str) -> Result<Vec<AiModelModel>>;
}

pub struct AiModelRepository {
    pool: DbPool,
}

impl AiModelRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TAiModelRepository for AiModelRepository {
    async fn list_active(&self) -> Result<Vec<AiModelModel>> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        ai_models::table
            .filter(ai_models::is_active.eq(true))
            .load::<AiModelModel>(&mut conn)
            .await
            .map_err(Error::from_std_error)
    }

    async fn list(&self) -> Result<Vec<AiModelModel>> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        ai_models::table
            .load::<AiModelModel>(&mut conn)
            .await
            .map_err(Error::from_std_error)
    }

    async fn get_by_provider_and_model_id(
        &self,
        provider: &AiProvider,
        model_id: &str,
    ) -> Result<Option<AiModelModel>> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        ai_models::table
            .filter(ai_models::provider.eq(provider))
            .filter(ai_models::model_id.eq(model_id))
            .first::<AiModelModel>(&mut conn)
            .await
            .optional()
            .map_err(Error::from_std_error)
    }

    async fn get(&self, id: Uuid) -> Result<Option<AiModelModel>> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        ai_models::table
            .find(id)
            .first::<AiModelModel>(&mut conn)
            .await
            .optional()
            .map_err(Error::from_std_error)
    }

    async fn enable_for_user(&self, user_id: &str, model_id: Uuid) -> Result<()> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        // Check if user_models table exists - we'll create migration for this
        // For now, this is a placeholder that will be implemented after migration
        diesel::sql_query(
            "INSERT INTO user_models (user_id, model_id, enabled, created_at, updated_at) 
             VALUES ($1, $2, true, NOW(), NOW())
             ON CONFLICT (user_id, model_id) 
             DO UPDATE SET enabled = true, updated_at = NOW()",
        )
        .bind::<diesel::sql_types::Text, _>(user_id)
        .bind::<diesel::sql_types::Uuid, _>(&model_id)
        .execute(&mut conn)
        .await
        .map_err(Error::from_std_error)?;

        Ok(())
    }

    async fn disable_for_user(&self, user_id: &str, model_id: Uuid) -> Result<()> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        diesel::sql_query(
            "UPDATE user_models 
             SET enabled = false, updated_at = NOW()
             WHERE user_id = $1 AND model_id = $2",
        )
        .bind::<diesel::sql_types::Text, _>(user_id)
        .bind::<diesel::sql_types::Uuid, _>(&model_id)
        .execute(&mut conn)
        .await
        .map_err(Error::from_std_error)?;

        Ok(())
    }

    async fn list_for_user(&self, _user_id: &str) -> Result<Vec<AiModelModel>> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        // TODO: Once user_models table exists, join with it to filter by user
        // For now, return all active models
        // This will be updated when the user_models migration is created
        ai_models::table
            .filter(ai_models::is_active.eq(true))
            .load::<AiModelModel>(&mut conn)
            .await
            .map_err(Error::from_std_error)
    }
}
