use async_trait::async_trait;
use diesel::prelude::*;
use diesel::OptionalExtension;
use diesel::QueryableByName;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use emixdiesel::{Error, Result};
use uuid::Uuid;

use crate::db::dto::{Pagination, ResultSet};
use crate::db::models::{CreateUserDto, NewUser, UpdateUser, UpdateUserDto, UserModel};
use crate::db::{schema::users, DbPool};

// Placeholder trait for FilterCondition - not currently used
pub trait FilterCondition<T>: Send + Sync {}

#[async_trait]
pub trait TUserRepository: Send + Sync {
    async fn list(
        &self,
        filter: Option<Box<dyn FilterCondition<UserModel> + Send + Sync>>,
        pagination: Option<Pagination>,
    ) -> Result<ResultSet<UserModel>>;
    async fn count(
        &self,
        filter: Option<Box<dyn FilterCondition<UserModel> + Send + Sync>>,
    ) -> Result<u64>;
    async fn get(&self, id: String) -> Result<Option<UserModel>>;
    async fn create(&self, model: NewUser) -> Result<UserModel>;
    async fn update(&self, id: String, model: UpdateUserDto) -> Result<UserModel>;
    async fn upsert(&self, model: CreateUserDto) -> Result<UserModel>;
    async fn delete(&self, id: String) -> Result<()>;
}

pub struct UserRepository {
    pool: DbPool,
}

impl UserRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    async fn resolve_role_id(conn: &mut AsyncPgConnection, role_name: &str) -> Result<Uuid> {
        #[derive(QueryableByName)]
        struct RoleIdRow {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
        }

        let normalized = role_name.trim().to_uppercase();

        let role = diesel::sql_query(
            "SELECT id FROM roles WHERE normalized_name = $1 OR name = $2 LIMIT 1",
        )
        .bind::<diesel::sql_types::Text, _>(&normalized)
        .bind::<diesel::sql_types::Text, _>(role_name.trim())
        .get_result::<RoleIdRow>(conn)
        .await
        .optional()
        .map_err(Error::from_std_error)?;

        role.map(|r| r.id)
            .ok_or_else(|| Error::from_other_error(format!("Role '{}' not found", role_name)))
    }
}

#[async_trait]
impl TUserRepository for UserRepository {
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

    async fn create(&self, model: NewUser) -> Result<UserModel> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        diesel::insert_into(users::table)
            .values(&model)
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
                users::name.eq(&new_user.name),
                users::username.eq(&new_user.username),
                users::avatar_url.eq(&new_user.avatar_url),
                users::email_verified.eq(&new_user.email_verified),
                users::updated_at.eq(chrono::Utc::now()),
            ))
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
}

// Extension methods for account management and roles
impl UserRepository {
    pub async fn get_by_email(&self, email: &str) -> Result<Option<UserModel>> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        // Case-insensitive lookup using normalized_email
        // Note: This uses raw SQL since normalized_email may not be in schema.rs yet
        // For now, fall back to case-insensitive email lookup
        let normalized_email = email.to_lowercase();

        // Try to find by email (case-insensitive)
        let result = users::table
            .filter(users::email.ilike(&normalized_email))
            .first::<UserModel>(&mut conn)
            .await
            .optional()
            .map_err(Error::from_std_error)?;

        Ok(result)
    }

    pub async fn get_by_username(&self, username: &str) -> Result<Option<UserModel>> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        // Case-insensitive lookup using normalized_username
        let normalized_username = username.to_lowercase();

        // Try to find by username (case-insensitive)
        let result = users::table
            .filter(users::username.ilike(&normalized_username))
            .first::<UserModel>(&mut conn)
            .await
            .optional()
            .map_err(Error::from_std_error)?;

        Ok(result)
    }

    pub async fn get_by_username_or_email(
        &self,
        username_or_email: &str,
    ) -> Result<Option<UserModel>> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        let normalized = username_or_email.to_lowercase();

        // Try to find by username or email (case-insensitive)
        let result = users::table
            .filter(
                users::username
                    .ilike(&normalized)
                    .or(users::email.ilike(&normalized)),
            )
            .first::<UserModel>(&mut conn)
            .await
            .optional()
            .map_err(Error::from_std_error)?;

        Ok(result)
    }

    pub async fn enable_user(&self, user_id: &str) -> Result<()> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        // Note: disabled field may not be in schema.rs yet, use raw SQL
        diesel::sql_query("UPDATE users SET disabled = false WHERE id = $1")
            .bind::<diesel::sql_types::Text, _>(user_id)
            .execute(&mut conn)
            .await
            .map_err(Error::from_std_error)?;

        Ok(())
    }

    pub async fn disable_user(&self, user_id: &str) -> Result<()> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        // Note: disabled field may not be in schema.rs yet, use raw SQL
        diesel::sql_query("UPDATE users SET disabled = true WHERE id = $1")
            .bind::<diesel::sql_types::Text, _>(user_id)
            .execute(&mut conn)
            .await
            .map_err(Error::from_std_error)?;

        Ok(())
    }

    pub async fn lock_user(&self, user_id: &str, duration_minutes: i32) -> Result<()> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        let lockout_end = chrono::Utc::now() + chrono::Duration::minutes(duration_minutes as i64);

        // Use raw SQL since lockout_end may not be in schema.rs yet
        diesel::sql_query("UPDATE users SET locked_out = true, lockout_end = $1 WHERE id = $2")
            .bind::<diesel::sql_types::Timestamptz, _>(&lockout_end)
            .bind::<diesel::sql_types::Text, _>(user_id)
            .execute(&mut conn)
            .await
            .map_err(Error::from_std_error)?;

        Ok(())
    }

    pub async fn unlock_user(&self, user_id: &str) -> Result<()> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        // Use raw SQL since locked_out and lockout_end may not be in schema.rs yet
        diesel::sql_query("UPDATE users SET locked_out = false, lockout_end = NULL WHERE id = $1")
            .bind::<diesel::sql_types::Text, _>(user_id)
            .execute(&mut conn)
            .await
            .map_err(Error::from_std_error)?;

        Ok(())
    }

    pub async fn increment_failed_login(&self, user_id: &str) -> Result<()> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        // Use raw SQL to increment and check for lockout
        const MAX_FAILED_ATTEMPTS: i32 = 5;
        const LOCKOUT_DURATION_MINUTES: i32 = 15;

        // Increment access_failed_count and lock if >= MAX_FAILED_ATTEMPTS
        diesel::sql_query(
            r#"
            UPDATE users 
            SET access_failed_count = access_failed_count + 1,
                locked_out = CASE 
                    WHEN access_failed_count + 1 >= $1 THEN true 
                    ELSE locked_out 
                END,
                lockout_end = CASE 
                    WHEN access_failed_count + 1 >= $1 THEN NOW() + ($2 || ' minutes')::INTERVAL
                    ELSE lockout_end 
                END
            WHERE id = $3
            "#,
        )
        .bind::<diesel::sql_types::Int4, _>(&MAX_FAILED_ATTEMPTS)
        .bind::<diesel::sql_types::Int4, _>(&LOCKOUT_DURATION_MINUTES)
        .bind::<diesel::sql_types::Text, _>(user_id)
        .execute(&mut conn)
        .await
        .map_err(Error::from_std_error)?;

        Ok(())
    }

    pub async fn reset_failed_login(&self, user_id: &str) -> Result<()> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        diesel::sql_query("UPDATE users SET access_failed_count = 0 WHERE id = $1")
            .bind::<diesel::sql_types::Text, _>(user_id)
            .execute(&mut conn)
            .await
            .map_err(Error::from_std_error)?;

        Ok(())
    }

    pub async fn update_last_login(&self, user_id: &str) -> Result<()> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        diesel::sql_query(
            "UPDATE users SET last_login_at = NOW(), login_count = login_count + 1 WHERE id = $1",
        )
        .bind::<diesel::sql_types::Text, _>(user_id)
        .execute(&mut conn)
        .await
        .map_err(Error::from_std_error)?;

        Ok(())
    }

    pub async fn get_user_roles(&self, user_id: &str) -> Result<Vec<String>> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        #[derive(QueryableByName)]
        struct RoleRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            role_name: String,
        }

        let roles: Vec<RoleRow> = diesel::sql_query(
            r#"
            SELECT r.name as role_name
            FROM user_roles ur
            INNER JOIN roles r ON ur.role_id = r.id
            WHERE ur.user_id = $1
            ORDER BY r.name
            "#,
        )
        .bind::<diesel::sql_types::Text, _>(user_id)
        .load(&mut conn)
        .await
        .map_err(Error::from_std_error)?;

        Ok(roles.into_iter().map(|r| r.role_name).collect())
    }

    pub async fn add_role(
        &self,
        user_id: &str,
        role_name: &str,
        assigned_by: Option<&str>,
    ) -> Result<()> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        let role_id = Self::resolve_role_id(&mut conn, role_name).await?;

        diesel::sql_query(
            "INSERT INTO user_roles (user_id, role_id, assigned_by) VALUES ($1, $2, $3) ON CONFLICT (user_id, role_id) DO NOTHING"
        )
        .bind::<diesel::sql_types::Text, _>(user_id)
        .bind::<diesel::sql_types::Uuid, _>(&role_id)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(assigned_by)
        .execute(&mut conn)
        .await
        .map_err(Error::from_std_error)?;

        Ok(())
    }

    pub async fn remove_role(&self, user_id: &str, role_name: &str) -> Result<()> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        let role_id = Self::resolve_role_id(&mut conn, role_name).await?;

        diesel::sql_query("DELETE FROM user_roles WHERE user_id = $1 AND role_id = $2")
            .bind::<diesel::sql_types::Text, _>(user_id)
            .bind::<diesel::sql_types::Uuid, _>(&role_id)
            .execute(&mut conn)
            .await
            .map_err(Error::from_std_error)?;

        Ok(())
    }

    pub async fn has_role(&self, user_id: &str, role_name: &str) -> Result<bool> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        let role_id = match Self::resolve_role_id(&mut conn, role_name).await {
            Ok(id) => id,
            Err(_) => {
                return Ok(false);
            }
        };

        #[derive(QueryableByName)]
        struct ExistsRow {
            #[diesel(sql_type = diesel::sql_types::Bool)]
            exists: bool,
        }

        let result: Vec<ExistsRow> = diesel::sql_query(
            "SELECT EXISTS(SELECT 1 FROM user_roles WHERE user_id = $1 AND role_id = $2) as exists",
        )
        .bind::<diesel::sql_types::Text, _>(user_id)
        .bind::<diesel::sql_types::Uuid, _>(&role_id)
        .load(&mut conn)
        .await
        .map_err(Error::from_std_error)?;

        Ok(result.into_iter().next().map(|r| r.exists).unwrap_or(false))
    }
}
