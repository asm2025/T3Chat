use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{NotSet, Set, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "messages")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub chat_id: Uuid,
    pub role: MessageRole,
    #[sea_orm(column_type = "Text")]
    pub content: String,
    #[sea_orm(column_type = "Json", nullable)]
    pub metadata: Option<serde_json::Value>,
    #[sea_orm(nullable)]
    pub parent_message_id: Option<Uuid>,
    pub sequence_number: i32,
    pub created_at: DateTime<Utc>,
    #[sea_orm(nullable)]
    pub tokens_used: Option<i32>,
    #[sea_orm(nullable)]
    pub model_used: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Text")]
pub enum MessageRole {
    #[sea_orm(string_value = "user")]
    User,
    #[sea_orm(string_value = "assistant")]
    Assistant,
    #[sea_orm(string_value = "system")]
    System,
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
            ..Default::default()
        }
    }

    async fn before_save<C>(mut self, _db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if insert {
            self.created_at = Set(Utc::now());
        }
        Ok(self)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateMessageDto {
    pub chat_id: Uuid,
    pub role: MessageRole,
    pub content: String,
    pub metadata: Option<serde_json::Value>,
    pub parent_message_id: Option<Uuid>,
    pub sequence_number: i32,
}

impl From<CreateMessageDto> for ActiveModel {
    fn from(dto: CreateMessageDto) -> Self {
        Self {
            id: NotSet,
            chat_id: Set(dto.chat_id),
            role: Set(dto.role),
            content: Set(dto.content),
            metadata: Set(dto.metadata),
            parent_message_id: Set(dto.parent_message_id),
            sequence_number: Set(dto.sequence_number),
            created_at: NotSet,
            tokens_used: Set(None),
            model_used: Set(None),
        }
    }
}

pub use ActiveModel as MessageModelDto;
pub use Column as MessageColumn;
pub use Entity as MessageEntity;
pub use Model as MessageModel;

