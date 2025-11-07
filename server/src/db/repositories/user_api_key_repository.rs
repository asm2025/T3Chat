use async_trait::async_trait;
use emixdb::{Error, Result, prelude::*};
use sea_orm::{DatabaseTransaction, TransactionTrait};
use uuid::Uuid;

use crate::db::schema::{
    AiProvider, CreateUserApiKeyDto, UpdateUserApiKeyDto, UserApiKeyEntity, UserApiKeyModel,
    UserApiKeyModelDto,
};

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
    db: sea_orm::DatabaseConnection,
}

impl UserApiKeyRepository {
    pub fn new(db: sea_orm::DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl IHasDatabase for UserApiKeyRepository {
    fn database(&self) -> &sea_orm::DatabaseConnection {
        &self.db
    }

    async fn begin_transaction(&self) -> Result<DatabaseTransaction> {
        self.db.begin().await.map_err(Error::from_std_error)
    }
}

#[async_trait]
impl IUserApiKeyRepository for UserApiKeyRepository {
    async fn list_by_user(&self, user_id: &str) -> Result<Vec<UserApiKeyModel>> {
        UserApiKeyEntity::find()
            .filter(crate::db::schema::UserApiKeyColumn::UserId.eq(user_id))
            .all(self.database())
            .await
            .map_err(Error::from_std_error)
    }

    async fn get_default_for_provider(
        &self,
        user_id: &str,
        provider: &AiProvider,
    ) -> Result<Option<UserApiKeyModel>> {
        UserApiKeyEntity::find()
            .filter(crate::db::schema::UserApiKeyColumn::UserId.eq(user_id))
            .filter(crate::db::schema::UserApiKeyColumn::Provider.eq(provider.clone()))
            .filter(crate::db::schema::UserApiKeyColumn::IsDefault.eq(true))
            .one(self.database())
            .await
            .map_err(Error::from_std_error)
    }

    async fn create(&self, model: CreateUserApiKeyDto) -> Result<UserApiKeyModel> {
        let active_model: UserApiKeyModelDto = model.into();
        active_model
            .insert(self.database())
            .await
            .map_err(Error::from_std_error)
    }

    async fn update(&self, id: Uuid, model: UpdateUserApiKeyDto) -> Result<UserApiKeyModel> {
        let existing = UserApiKeyEntity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(Error::from_std_error)?
            .ok_or_else(|| sea_orm::DbErr::RecordNotFound("User API key not found".to_owned()))
            .map_err(Error::from_std_error)?;
        let mut active_model: UserApiKeyModelDto = existing.into();
        model.merge(&mut active_model);
        active_model
            .update(self.database())
            .await
            .map_err(Error::from_std_error)
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        UserApiKeyEntity::delete_by_id(id)
            .exec(self.database())
            .await
            .map_err(Error::from_std_error)?;
        Ok(())
    }

    async fn set_default(&self, id: Uuid, user_id: &str) -> Result<()> {
        let txn = self.begin_transaction().await?;

        // First, unset all defaults for this user and provider
        let key = UserApiKeyEntity::find_by_id(id)
            .one(&txn)
            .await
            .map_err(Error::from_std_error)?
            .ok_or_else(|| sea_orm::DbErr::RecordNotFound("User API key not found".to_owned()))
            .map_err(Error::from_std_error)?;

        let provider = key.provider.clone();
        let mut keys = UserApiKeyEntity::find()
            .filter(crate::db::schema::UserApiKeyColumn::UserId.eq(user_id))
            .filter(crate::db::schema::UserApiKeyColumn::Provider.eq(provider))
            .all(&txn)
            .await
            .map_err(Error::from_std_error)?;

        for key in &mut keys {
            let mut active_model: UserApiKeyModelDto = key.clone().into();
            active_model.is_default = sea_orm::Set(false);
            active_model.update(&txn).await.map_err(Error::from_std_error)?;
        }

        // Then set this one as default
        let mut active_model: UserApiKeyModelDto = key.into();
        active_model.is_default = sea_orm::Set(true);
        active_model.update(&txn).await.map_err(Error::from_std_error)?;

        txn.commit().await.map_err(Error::from_std_error)?;
        Ok(())
    }
}

