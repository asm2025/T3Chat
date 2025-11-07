use async_trait::async_trait;
use chrono::{DateTime, Utc};
use emixdb::schema::Merge;
use sea_orm::{NotSet, Set, prelude::*};
use serde::{Deserialize, Serialize};
use crate::db::schema::ai_model::AiProvider;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "chats")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub user_id: String,
    pub title: String,
    pub model_provider: AiProvider,
    pub model_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[sea_orm(nullable)]
    pub deleted_at: Option<DateTime<Utc>>,
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
            deleted_at: Set(None),
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
pub struct CreateChatDto {
    pub user_id: String,
    pub title: String,
    pub model_provider: AiProvider,
    pub model_id: String,
}

impl From<CreateChatDto> for ActiveModel {
    fn from(dto: CreateChatDto) -> Self {
        Self {
            id: NotSet,
            user_id: Set(dto.user_id),
            title: Set(dto.title),
            model_provider: Set(dto.model_provider),
            model_id: Set(dto.model_id),
            created_at: NotSet,
            updated_at: NotSet,
            deleted_at: Set(None),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateChatDto {
    pub title: Option<String>,
    pub model_provider: Option<AiProvider>,
    pub model_id: Option<String>,
}

impl Merge<ActiveModel> for UpdateChatDto {
    fn merge(&self, model: &mut ActiveModel) -> bool {
        let mut changed = false;

        if let Some(title) = &self.title {
            model.title = Set(title.clone());
            changed = true;
        }

        if let Some(model_provider) = &self.model_provider {
            model.model_provider = Set(model_provider.clone());
            changed = true;
        }

        if let Some(model_id) = &self.model_id {
            model.model_id = Set(model_id.clone());
            changed = true;
        }

        changed
    }
}

pub use ActiveModel as ChatModelDto;
pub use Column as ChatColumn;
pub use Entity as ChatEntity;
pub use Model as ChatModel;

