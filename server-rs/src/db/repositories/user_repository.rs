use async_trait::async_trait;
use emixdb::{Error, Result, prelude::*};
use sea_orm::{DatabaseTransaction, PaginatorTrait, TransactionTrait};

use crate::db::schema::{CreateUserDto, UpdateUserDto, UserEntity, UserModel, UserModelDto};

#[async_trait]
pub trait IUserRepository: IRepository<UserEntity, UpdateUserDto> + Send + Sync {
    async fn upsert(&self, model: CreateUserDto) -> Result<UserModel>;
}

pub struct UserRepository {
    db: DatabaseConnection,
}

impl UserRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl IHasDatabase for UserRepository {
    fn database(&self) -> &DatabaseConnection {
        &self.db
    }

    async fn begin_transaction(&self) -> Result<DatabaseTransaction> {
        self.db.begin().await.map_err(Error::from_std_error)
    }
}

#[async_trait]
impl IRepository<UserEntity, UpdateUserDto> for UserRepository {
    async fn list(
        &self,
        filter: Option<Box<dyn FilterCondition<UserEntity> + Send + Sync>>,
        pagination: Option<Pagination>,
    ) -> Result<ResultSet<UserModel>> {
        let mut query = <UserEntity as EntityTrait>::find();

        if let Some(f) = &filter {
            query = f.apply(query);
        }

        let total = query
            .clone()
            .count(self.database())
            .await
            .map_err(Error::from_std_error)?;

        if let Some(p) = pagination {
            query = query.offset((p.page - 1) * p.page_size).limit(p.page_size);
        }

        let data = query
            .all(self.database())
            .await
            .map_err(Error::from_std_error)?;

        Ok(ResultSet {
            data,
            total,
            pagination,
        })
    }

    async fn count(
        &self,
        filter: Option<Box<dyn FilterCondition<UserEntity> + Send + Sync>>,
    ) -> Result<u64> {
        let mut query = <UserEntity as EntityTrait>::find();

        if let Some(f) = &filter {
            query = f.apply(query);
        }

        query
            .clone()
            .count(self.database())
            .await
            .map_err(Error::from_std_error)
    }

    async fn get(&self, id: String) -> Result<Option<UserModel>> {
        UserEntity::find_by_id(id)
            .one(self.database())
            .await
            .map_err(Error::from_std_error)
    }

    async fn create(
        &self,
        model: <UserEntity as EntityTrait>::Model,
    ) -> Result<<UserEntity as EntityTrait>::Model> {
        let active_model: <UserEntity as EntityTrait>::ActiveModel = model.into();
        active_model
            .insert(self.database())
            .await
            .map_err(Error::from_std_error)
    }

    async fn update(&self, id: String, model: UpdateUserDto) -> Result<UserModel> {
        let existing = UserEntity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(Error::from_std_error)?
            .ok_or_else(|| sea_orm::DbErr::RecordNotFound("User not found".to_owned()))
            .map_err(Error::from_std_error)?;
        let mut active_model: UserModelDto = existing.into();
        model.merge(&mut active_model);
        active_model
            .update(self.database())
            .await
            .map_err(Error::from_std_error)
    }

    async fn delete(&self, id: String) -> Result<()> {
        UserEntity::delete_by_id(id)
            .exec(self.database())
            .await
            .map_err(Error::from_std_error)?;
        Ok(())
    }
}

#[async_trait]
impl IUserRepository for UserRepository {
    async fn upsert(&self, model: CreateUserDto) -> Result<UserModel> {
        match UserEntity::find_by_id(model.id.clone())
            .one(&self.db)
            .await
            .map_err(Error::from_std_error)
        {
            Ok(Some(existing)) => {
                let mut active_model: UserModelDto = existing.into();
                model.merge(&mut active_model);
                active_model
                    .update(self.database())
                    .await
                    .map_err(Error::from_std_error)
            }
            Ok(None) => self
                .create(model.into())
                .await
                .map_err(Error::from_std_error),
            Err(e) => Err(e),
        }
    }
}
