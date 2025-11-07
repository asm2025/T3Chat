use async_trait::async_trait;
use emixdb::{Error, Result, prelude::*};
use sea_orm::{DatabaseTransaction, TransactionTrait};
use uuid::Uuid;

use crate::db::schema::{
    ChatEntity, ChatModel, ChatModelDto, CreateChatDto, UpdateChatDto,
};

#[async_trait]
pub trait IChatRepository: Send + Sync {
    async fn list_by_user(
        &self,
        user_id: &str,
        pagination: Option<Pagination>,
    ) -> Result<ResultSet<ChatModel>>;
    async fn get_by_id(&self, id: Uuid, user_id: &str) -> Result<Option<ChatModel>>;
    async fn create(&self, model: CreateChatDto) -> Result<ChatModel>;
    async fn update(&self, id: Uuid, user_id: &str, model: UpdateChatDto) -> Result<ChatModel>;
    async fn delete(&self, id: Uuid, user_id: &str) -> Result<()>;
}

pub struct ChatRepository {
    db: sea_orm::DatabaseConnection,
}

impl ChatRepository {
    pub fn new(db: sea_orm::DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl IHasDatabase for ChatRepository {
    fn database(&self) -> &sea_orm::DatabaseConnection {
        &self.db
    }

    async fn begin_transaction(&self) -> Result<DatabaseTransaction> {
        self.db.begin().await.map_err(Error::from_std_error)
    }
}

#[async_trait]
impl IChatRepository for ChatRepository {
    async fn list_by_user(
        &self,
        user_id: &str,
        pagination: Option<Pagination>,
    ) -> Result<ResultSet<ChatModel>> {
        let mut query = ChatEntity::find()
            .filter(crate::db::schema::ChatColumn::UserId.eq(user_id))
            .filter(crate::db::schema::ChatColumn::DeletedAt.is_null());

        let total = query
            .clone()
            .count(self.database())
            .await
            .map_err(Error::from_std_error)?;

        if let Some(p) = pagination {
            query = query
                .offset((p.page - 1) * p.page_size)
                .limit(p.page_size);
        }

        let data = query
            .order_by_desc(crate::db::schema::ChatColumn::UpdatedAt)
            .all(self.database())
            .await
            .map_err(Error::from_std_error)?;

        Ok(ResultSet {
            data,
            total,
            pagination,
        })
    }

    async fn get_by_id(&self, id: Uuid, user_id: &str) -> Result<Option<ChatModel>> {
        ChatEntity::find()
            .filter(crate::db::schema::ChatColumn::Id.eq(id))
            .filter(crate::db::schema::ChatColumn::UserId.eq(user_id))
            .filter(crate::db::schema::ChatColumn::DeletedAt.is_null())
            .one(self.database())
            .await
            .map_err(Error::from_std_error)
    }

    async fn create(&self, model: CreateChatDto) -> Result<ChatModel> {
        let active_model: ChatModelDto = model.into();
        active_model
            .insert(self.database())
            .await
            .map_err(Error::from_std_error)
    }

    async fn update(&self, id: Uuid, user_id: &str, model: UpdateChatDto) -> Result<ChatModel> {
        let existing = ChatEntity::find()
            .filter(crate::db::schema::ChatColumn::Id.eq(id))
            .filter(crate::db::schema::ChatColumn::UserId.eq(user_id))
            .filter(crate::db::schema::ChatColumn::DeletedAt.is_null())
            .one(&self.db)
            .await
            .map_err(Error::from_std_error)?
            .ok_or_else(|| sea_orm::DbErr::RecordNotFound("Chat not found".to_owned()))
            .map_err(Error::from_std_error)?;
        let mut active_model: ChatModelDto = existing.into();
        model.merge(&mut active_model);
        active_model
            .update(self.database())
            .await
            .map_err(Error::from_std_error)
    }

    async fn delete(&self, id: Uuid, user_id: &str) -> Result<()> {
        let existing = ChatEntity::find()
            .filter(crate::db::schema::ChatColumn::Id.eq(id))
            .filter(crate::db::schema::ChatColumn::UserId.eq(user_id))
            .filter(crate::db::schema::ChatColumn::DeletedAt.is_null())
            .one(&self.db)
            .await
            .map_err(Error::from_std_error)?
            .ok_or_else(|| sea_orm::DbErr::RecordNotFound("Chat not found".to_owned()))
            .map_err(Error::from_std_error)?;
        let mut active_model: ChatModelDto = existing.into();
        active_model.deleted_at = sea_orm::Set(Some(chrono::Utc::now()));
        active_model
            .update(self.database())
            .await
            .map_err(Error::from_std_error)?;
        Ok(())
    }
}

