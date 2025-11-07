use async_trait::async_trait;
use emixdb::{Error, Result, prelude::*};
use sea_orm::{DatabaseTransaction, TransactionTrait};
use uuid::Uuid;

use crate::db::schema::{AiModelEntity, AiModelModel, AiProvider};

#[async_trait]
pub trait IAiModelRepository: Send + Sync {
    async fn list_active(&self) -> Result<Vec<AiModelModel>>;
    async fn get_by_provider_and_model_id(
        &self,
        provider: &AiProvider,
        model_id: &str,
    ) -> Result<Option<AiModelModel>>;
    async fn get_by_id(&self, id: Uuid) -> Result<Option<AiModelModel>>;
}

pub struct AiModelRepository {
    db: sea_orm::DatabaseConnection,
}

impl AiModelRepository {
    pub fn new(db: sea_orm::DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl IHasDatabase for AiModelRepository {
    fn database(&self) -> &sea_orm::DatabaseConnection {
        &self.db
    }

    async fn begin_transaction(&self) -> Result<DatabaseTransaction> {
        self.db.begin().await.map_err(Error::from_std_error)
    }
}

#[async_trait]
impl IAiModelRepository for AiModelRepository {
    async fn list_active(&self) -> Result<Vec<AiModelModel>> {
        AiModelEntity::find()
            .filter(crate::db::schema::AiModelColumn::IsActive.eq(true))
            .all(self.database())
            .await
            .map_err(Error::from_std_error)
    }

    async fn get_by_provider_and_model_id(
        &self,
        provider: &AiProvider,
        model_id: &str,
    ) -> Result<Option<AiModelModel>> {
        AiModelEntity::find()
            .filter(crate::db::schema::AiModelColumn::Provider.eq(provider.clone()))
            .filter(crate::db::schema::AiModelColumn::ModelId.eq(model_id))
            .one(self.database())
            .await
            .map_err(Error::from_std_error)
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Option<AiModelModel>> {
        AiModelEntity::find_by_id(id)
            .one(self.database())
            .await
            .map_err(Error::from_std_error)
    }
}

