use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl, scoped_futures::ScopedFutureExt};
use emixdiesel::{Error, Result};
use uuid::Uuid;

use crate::db::models::{
    AiProvider, CreateUserApiKeyDto, NewUserApiKey, UpdateUserApiKey, UpdateUserApiKeyDto,
    UserApiKeyModel,
};
use crate::db::{DbPool, schema::user_api_keys};

#[async_trait]
pub trait IUserApiKeyRepository: Send + Sync {
    async fn list_by_user(&self, user_id: &str) -> Result<Vec<UserApiKeyModel>>;
    async fn get_default_for_provider(
        &self,
        user_id: &str,
        provider: &AiProvider,
    ) -> Result<Option<UserApiKeyModel>>;
    async fn create(&self, model: CreateUserApiKeyDto) -> Result<UserApiKeyModel>;
    async fn update(&self, id: Uuid, model: UpdateUserApiKeyDto) -> Result<UserApiKeyModel>;
    async fn delete(&self, id: Uuid) -> Result<()>;
    async fn set_default(&self, id: Uuid, user_id: &str) -> Result<()>;
}

pub struct UserApiKeyRepository {
    pool: DbPool,
}

impl UserApiKeyRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl IUserApiKeyRepository for UserApiKeyRepository {
    async fn list_by_user(&self, user_id: &str) -> Result<Vec<UserApiKeyModel>> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        user_api_keys::table
            .filter(user_api_keys::user_id.eq(user_id))
            .load::<UserApiKeyModel>(&mut conn)
            .await
            .map_err(Error::from_std_error)
    }

    async fn get_default_for_provider(
        &self,
        user_id: &str,
        provider: &AiProvider,
    ) -> Result<Option<UserApiKeyModel>> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        user_api_keys::table
            .filter(user_api_keys::user_id.eq(user_id))
            .filter(user_api_keys::provider.eq(provider))
            .filter(user_api_keys::is_default.eq(true))
            .first::<UserApiKeyModel>(&mut conn)
            .await
            .optional()
            .map_err(Error::from_std_error)
    }

    async fn create(&self, model: CreateUserApiKeyDto) -> Result<UserApiKeyModel> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        let new_key: NewUserApiKey = model.into();

        diesel::insert_into(user_api_keys::table)
            .values(&new_key)
            .get_result(&mut conn)
            .await
            .map_err(Error::from_std_error)
    }

    async fn update(&self, id: Uuid, model: UpdateUserApiKeyDto) -> Result<UserApiKeyModel> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        // Check if key exists
        let _existing = user_api_keys::table
            .find(id)
            .first::<UserApiKeyModel>(&mut conn)
            .await
            .optional()
            .map_err(Error::from_std_error)?
            .ok_or_else(|| Error::from_other_error("User API key not found".to_string()))?;

        let update_key: UpdateUserApiKey = model.into();

        diesel::update(user_api_keys::table.find(id))
            .set(&update_key)
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

        diesel::delete(user_api_keys::table.find(id))
            .execute(&mut conn)
            .await
            .map_err(Error::from_std_error)?;

        Ok(())
    }

    async fn set_default(&self, id: Uuid, user_id: &str) -> Result<()> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            async move {
                // Get the key to set as default
                let key = user_api_keys::table
                    .find(id)
                    .first::<UserApiKeyModel>(conn)
                    .await
                    .optional()?
                    .ok_or_else(|| diesel::result::Error::NotFound)?;

                let provider = key.provider;

                // Unset all defaults for this user and provider
                diesel::update(
                    user_api_keys::table
                        .filter(user_api_keys::user_id.eq(user_id))
                        .filter(user_api_keys::provider.eq(provider)),
                )
                .set(user_api_keys::is_default.eq(false))
                .execute(conn)
                .await?;

                // Set this one as default
                diesel::update(user_api_keys::table.find(id))
                    .set((
                        user_api_keys::is_default.eq(true),
                        user_api_keys::updated_at.eq(chrono::Utc::now()),
                    ))
                    .execute(conn)
                    .await?;

                Ok(())
            }
            .scope_boxed()
        })
        .await
        .map_err(Error::from_std_error)
    }
}
