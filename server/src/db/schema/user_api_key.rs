use async_trait::async_trait;
use chrono::{DateTime, Utc};
use emixdb::schema::Merge;
use sea_orm::{NotSet, Set, prelude::*};
use serde::{Deserialize, Serialize};
use crate::db::schema::ai_model::AiProvider;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user_api_keys")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub user_id: String,
    pub provider: AiProvider,
    #[sea_orm(column_type = "Text")]
    pub encrypted_key: String,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        let now = Utc::now();
        Self {
            id: Set(Uuid::new_v4()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
    }

    async fn before_save<C>(mut self, _db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        let now = Utc::now();

        if insert {
            self.created_at = Set(now);
        }

        self.updated_at = Set(now);
        Ok(self)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserApiKeyDto {
    pub user_id: String,
    pub provider: AiProvider,
    pub encrypted_key: String,
    pub is_default: bool,
}

impl From<CreateUserApiKeyDto> for ActiveModel {
    fn from(dto: CreateUserApiKeyDto) -> Self {
        Self {
            id: NotSet,
            user_id: Set(dto.user_id),
            provider: Set(dto.provider),
            encrypted_key: Set(dto.encrypted_key),
            is_default: Set(dto.is_default),
            created_at: NotSet,
            updated_at: NotSet,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserApiKeyDto {
    pub encrypted_key: Option<String>,
    pub is_default: Option<bool>,
}

impl Merge<ActiveModel> for UpdateUserApiKeyDto {
    fn merge(&self, model: &mut ActiveModel) -> bool {
        let mut changed = false;

        if let Some(encrypted_key) = &self.encrypted_key {
            model.encrypted_key = Set(encrypted_key.clone());
            changed = true;
        }

        if let Some(is_default) = &self.is_default {
            model.is_default = Set(*is_default);
            changed = true;
        }

        changed
    }
}

pub use ActiveModel as UserApiKeyModelDto;
pub use Column as UserApiKeyColumn;
pub use Entity as UserApiKeyEntity;
pub use Model as UserApiKeyModel;

