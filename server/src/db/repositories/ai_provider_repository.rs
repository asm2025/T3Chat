use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use emixdiesel::{Error, Result};
use uuid::Uuid;

use crate::db::dto::{Pagination, ResultSet};
use crate::db::models::ai_provider::{AiProvider, NewAiProvider, UpdateAiProvider};
use crate::db::{DbPool, schema::ai_providers};

#[async_trait]
pub trait TAiProviderRepository: Send + Sync {
    async fn list(&self, pagination: Option<Pagination>) -> Result<ResultSet<AiProvider>>;
    async fn count(&self) -> Result<u64>;
    async fn get(&self, id: Uuid) -> Result<Option<AiProvider>>;
    async fn get_by_provider_id(&self, provider_id: &str) -> Result<Option<AiProvider>>;
    async fn create(&self, model: NewAiProvider) -> Result<AiProvider>;
    async fn update(&self, id: Uuid, model: UpdateAiProvider) -> Result<AiProvider>;
    async fn delete(&self, id: Uuid) -> Result<()>;
    async fn enable(&self, id: Uuid) -> Result<()>;
    async fn disable(&self, id: Uuid) -> Result<()>;
}

pub struct AiProviderRepository {
    pool: DbPool,
}

impl AiProviderRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TAiProviderRepository for AiProviderRepository {
    async fn list(&self, pagination: Option<Pagination>) -> Result<ResultSet<AiProvider>> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        // Count total records
        let total = ai_providers::table
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(Error::from_std_error)? as u64;

        // Apply pagination
        let mut query = ai_providers::table.into_boxed();

        if let Some(p) = pagination {
            query = query
                .offset(((p.page - 1) * p.page_size) as i64)
                .limit(p.page_size as i64);
        }

        let data = query
            .load::<AiProvider>(&mut conn)
            .await
            .map_err(Error::from_std_error)?;

        Ok(ResultSet {
            data,
            total,
            pagination,
        })
    }

    async fn count(&self) -> Result<u64> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        ai_providers::table
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map(|c| c as u64)
            .map_err(Error::from_std_error)
    }

    async fn get(&self, id: Uuid) -> Result<Option<AiProvider>> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        ai_providers::table
            .find(id)
            .first::<AiProvider>(&mut conn)
            .await
            .optional()
            .map_err(Error::from_std_error)
    }

    async fn get_by_provider_id(&self, provider_id: &str) -> Result<Option<AiProvider>> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        ai_providers::table
            .filter(ai_providers::provider_id.eq(provider_id))
            .first::<AiProvider>(&mut conn)
            .await
            .optional()
            .map_err(Error::from_std_error)
    }

    async fn create(&self, model: NewAiProvider) -> Result<AiProvider> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        diesel::insert_into(ai_providers::table)
            .values(&model)
            .get_result(&mut conn)
            .await
            .map_err(Error::from_std_error)
    }

    async fn update(&self, id: Uuid, model: UpdateAiProvider) -> Result<AiProvider> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        // Check if provider exists
        let _existing = ai_providers::table
            .find(&id)
            .first::<AiProvider>(&mut conn)
            .await
            .optional()
            .map_err(Error::from_std_error)?
            .ok_or_else(|| Error::NotFound("Provider not found".to_string()))?;

        diesel::update(ai_providers::table.find(&id))
            .set(&model)
            .get_result(&mut conn)
            .await
            .map_err(Error::from_std_error)
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        diesel::delete(ai_providers::table.find(id))
            .execute(&mut conn)
            .await
            .map_err(Error::from_std_error)?;

        Ok(())
    }

    async fn enable(&self, id: Uuid) -> Result<()> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        diesel::update(ai_providers::table.find(&id))
            .set(ai_providers::disabled.eq(false))
            .execute(&mut conn)
            .await
            .map_err(Error::from_std_error)?;

        Ok(())
    }

    async fn disable(&self, id: Uuid) -> Result<()> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        diesel::update(ai_providers::table.find(&id))
            .set(ai_providers::disabled.eq(true))
            .execute(&mut conn)
            .await
            .map_err(Error::from_std_error)?;

        Ok(())
    }
}
