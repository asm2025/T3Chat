use async_trait::async_trait;
use emixdb::{Error, Result, prelude::*};
use sea_orm::{DatabaseTransaction, TransactionTrait};
use uuid::Uuid;

use crate::db::schema::{CreateMessageDto, MessageEntity, MessageModel, MessageModelDto};

#[async_trait]
pub trait IMessageRepository: Send + Sync {
    async fn list_by_chat(&self, chat_id: Uuid, user_id: &str) -> Result<Vec<MessageModel>>;
    async fn create(&self, model: CreateMessageDto) -> Result<MessageModel>;
    async fn get_next_sequence_number(&self, chat_id: Uuid) -> Result<i32>;
    async fn update_tokens_used(&self, id: Uuid, tokens: i32, model: &str) -> Result<()>;
}

pub struct MessageRepository {
    db: sea_orm::DatabaseConnection,
}

impl MessageRepository {
    pub fn new(db: sea_orm::DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl IHasDatabase for MessageRepository {
    fn database(&self) -> &sea_orm::DatabaseConnection {
        &self.db
    }

    async fn begin_transaction(&self) -> Result<DatabaseTransaction> {
        self.db.begin().await.map_err(Error::from_std_error)
    }
}

#[async_trait]
impl IMessageRepository for MessageRepository {
    async fn list_by_chat(&self, chat_id: Uuid, user_id: &str) -> Result<Vec<MessageModel>> {
        // Verify chat belongs to user
        let _chat = crate::db::schema::ChatEntity::find()
            .filter(crate::db::schema::ChatColumn::Id.eq(chat_id))
            .filter(crate::db::schema::ChatColumn::UserId.eq(user_id))
            .one(self.database())
            .await
            .map_err(Error::from_std_error)?
            .ok_or_else(|| sea_orm::DbErr::RecordNotFound("Chat not found".to_owned()))
            .map_err(Error::from_std_error)?;

        MessageEntity::find()
            .filter(crate::db::schema::MessageColumn::ChatId.eq(chat_id))
            .order_by_asc(crate::db::schema::MessageColumn::SequenceNumber)
            .all(self.database())
            .await
            .map_err(Error::from_std_error)
    }

    async fn create(&self, model: CreateMessageDto) -> Result<MessageModel> {
        let active_model: MessageModelDto = model.into();
        active_model
            .insert(self.database())
            .await
            .map_err(Error::from_std_error)
    }

    async fn get_next_sequence_number(&self, chat_id: Uuid) -> Result<i32> {
        use sea_orm::{QuerySelect, prelude::Expr};
        
        let max_seq = MessageEntity::find()
            .filter(crate::db::schema::MessageColumn::ChatId.eq(chat_id))
            .select_only()
            .column_as(
                Expr::col(crate::db::schema::MessageColumn::SequenceNumber).max(),
                "max_seq",
            )
            .into_tuple::<Option<i32>>()
            .one(self.database())
            .await
            .map_err(Error::from_std_error)?;

        Ok(max_seq.and_then(|x| x).unwrap_or(0) + 1)
    }

    async fn update_tokens_used(&self, id: Uuid, tokens: i32, model: &str) -> Result<()> {
        let existing = MessageEntity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(Error::from_std_error)?
            .ok_or_else(|| sea_orm::DbErr::RecordNotFound("Message not found".to_owned()))
            .map_err(Error::from_std_error)?;
        let mut active_model: MessageModelDto = existing.into();
        active_model.tokens_used = sea_orm::Set(Some(tokens));
        active_model.model_used = sea_orm::Set(Some(model.to_string()));
        active_model
            .update(self.database())
            .await
            .map_err(Error::from_std_error)?;
        Ok(())
    }
}

