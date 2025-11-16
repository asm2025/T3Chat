use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use emixdiesel::{Error, Result};
use uuid::Uuid;

use crate::db::models::{AiModelModel, AiProvider};
use crate::db::{DbPool, schema::ai_models};

#[async_trait]
pub trait IAiModelRepository: Send + Sync {
    async fn list_active(&self) -> Result<Vec<AiModelModel>>;
    async fn list_all(&self) -> Result<Vec<AiModelModel>>;
    async fn get_by_provider_and_model_id(
        &self,
        provider: &AiProvider,
        model_id: &str,
    ) -> Result<Option<AiModelModel>>;
    async fn get_by_id(&self, id: Uuid) -> Result<Option<AiModelModel>>;
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
impl IAiModelRepository for AiModelRepository {
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

    async fn list_all(&self) -> Result<Vec<AiModelModel>> {
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

    async fn get_by_id(&self, id: Uuid) -> Result<Option<AiModelModel>> {
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
}
