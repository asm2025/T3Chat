use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use emixdiesel::{Error, Result};

use crate::db::dto::{Pagination, ResultSet};
use crate::db::models::{CreateUserDto, NewUser, UpdateUser, UpdateUserDto, UserModel};
use crate::db::{DbPool, schema::users};

// Placeholder trait for FilterCondition - not currently used
pub trait FilterCondition<T>: Send + Sync {}

#[async_trait]
pub trait IUserRepository: Send + Sync {
    async fn upsert(&self, model: CreateUserDto) -> Result<UserModel>;
    async fn get(&self, id: String) -> Result<Option<UserModel>>;
    async fn update(&self, id: String, model: UpdateUserDto) -> Result<UserModel>;
    async fn delete(&self, id: String) -> Result<()>;
    async fn list(
        &self,
        filter: Option<Box<dyn FilterCondition<UserModel> + Send + Sync>>,
        pagination: Option<Pagination>,
    ) -> Result<ResultSet<UserModel>>;
    async fn count(
        &self,
        filter: Option<Box<dyn FilterCondition<UserModel> + Send + Sync>>,
    ) -> Result<u64>;
    async fn create(&self, model: UserModel) -> Result<UserModel>;
}

pub struct UserRepository {
    pool: DbPool,
}

impl UserRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl IUserRepository for UserRepository {
    async fn list(
        &self,
        _filter: Option<Box<dyn FilterCondition<UserModel> + Send + Sync>>,
        pagination: Option<Pagination>,
    ) -> Result<ResultSet<UserModel>> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        // Count total records
        let total = users::table
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(Error::from_std_error)? as u64;

        // Apply pagination
        let mut query = users::table.into_boxed();

        if let Some(p) = pagination {
            query = query
                .offset(((p.page - 1) * p.page_size) as i64)
                .limit(p.page_size as i64);
        }

        let data = query
            .load::<UserModel>(&mut conn)
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
        _filter: Option<Box<dyn FilterCondition<UserModel> + Send + Sync>>,
    ) -> Result<u64> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        users::table
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map(|c| c as u64)
            .map_err(Error::from_std_error)
    }

    async fn get(&self, id: String) -> Result<Option<UserModel>> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        users::table
            .find(id)
            .first::<UserModel>(&mut conn)
            .await
            .optional()
            .map_err(Error::from_std_error)
    }

    async fn create(&self, model: UserModel) -> Result<UserModel> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        let new_user = NewUser {
            id: model.id,
            email: model.email,
            display_name: model.display_name,
            image_url: model.image_url,
            created_at: model.created_at,
            updated_at: model.updated_at,
        };

        diesel::insert_into(users::table)
            .values(&new_user)
            .get_result(&mut conn)
            .await
            .map_err(Error::from_std_error)
    }

    async fn update(&self, id: String, model: UpdateUserDto) -> Result<UserModel> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        // Check if user exists
        let _existing = users::table
            .find(&id)
            .first::<UserModel>(&mut conn)
            .await
            .optional()
            .map_err(Error::from_std_error)?
            .ok_or_else(|| Error::from_other_error("User not found".to_string()))?;

        let update_user: UpdateUser = model.into();

        diesel::update(users::table.find(&id))
            .set(&update_user)
            .get_result(&mut conn)
            .await
            .map_err(Error::from_std_error)
    }

    async fn delete(&self, id: String) -> Result<()> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        diesel::delete(users::table.find(id))
            .execute(&mut conn)
            .await
            .map_err(Error::from_std_error)?;

        Ok(())
    }

    async fn upsert(&self, model: CreateUserDto) -> Result<UserModel> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        let new_user: NewUser = model.into();

        // Use PostgreSQL's ON CONFLICT for atomic upsert
        diesel::insert_into(users::table)
            .values(&new_user)
            .on_conflict(users::id)
            .do_update()
            .set((
                users::display_name.eq(&new_user.display_name),
                users::image_url.eq(&new_user.image_url),
                users::updated_at.eq(&new_user.updated_at),
            ))
            .get_result(&mut conn)
            .await
            .map_err(Error::from_std_error)
    }
}
