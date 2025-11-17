use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use emixdiesel::{Error, Result};

use crate::db::models::{
    CreateUserFeatureDto, Feature, NewUserFeature, UpdateUserFeature, UpdateUserFeatureDto,
    UserFeatureModel,
};
use crate::db::{DbPool, schema::user_features};

#[async_trait]
pub trait TUserFeatureRepository: Send + Sync {
    async fn list(&self, user_id: &str) -> Result<Vec<UserFeatureModel>>;
    async fn get(
        &self,
        user_id: &str,
        feature: &Feature,
    ) -> Result<Option<UserFeatureModel>>;
    async fn upsert(&self, model: CreateUserFeatureDto) -> Result<UserFeatureModel>;
    async fn update(&self, user_id: &str, feature: &Feature, model: UpdateUserFeatureDto) -> Result<UserFeatureModel>;
}

pub struct UserFeatureRepository {
    pool: DbPool,
}

impl UserFeatureRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TUserFeatureRepository for UserFeatureRepository {
    async fn list(&self, user_id: &str) -> Result<Vec<UserFeatureModel>> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        user_features::table
            .filter(user_features::user_id.eq(user_id))
            .load::<UserFeatureModel>(&mut conn)
            .await
            .map_err(Error::from_std_error)
    }

    async fn get(
        &self,
        user_id: &str,
        feature: &Feature,
    ) -> Result<Option<UserFeatureModel>> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        user_features::table
            .filter(user_features::user_id.eq(user_id))
            .filter(user_features::feature.eq(feature))
            .first::<UserFeatureModel>(&mut conn)
            .await
            .optional()
            .map_err(Error::from_std_error)
    }

    async fn upsert(&self, model: CreateUserFeatureDto) -> Result<UserFeatureModel> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        let new_feature: NewUserFeature = model.into();

        // Use PostgreSQL's ON CONFLICT for atomic upsert
        diesel::insert_into(user_features::table)
            .values(&new_feature)
            .on_conflict((user_features::user_id, user_features::feature))
            .do_update()
            .set((
                user_features::enabled.eq(&new_feature.enabled),
                user_features::updated_at.eq(&new_feature.updated_at),
            ))
            .get_result(&mut conn)
            .await
            .map_err(Error::from_std_error)
    }

    async fn update(&self, user_id: &str, feature: &Feature, model: UpdateUserFeatureDto) -> Result<UserFeatureModel> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::from_std_error(e))?;

        // Check if feature exists
        let _existing = user_features::table
            .filter(user_features::user_id.eq(user_id))
            .filter(user_features::feature.eq(feature))
            .first::<UserFeatureModel>(&mut conn)
            .await
            .optional()
            .map_err(Error::from_std_error)?
            .ok_or_else(|| Error::from_other_error("User feature not found".to_string()))?;

        let update_feature: UpdateUserFeature = model.into();

        diesel::update(
            user_features::table
                .filter(user_features::user_id.eq(user_id))
                .filter(user_features::feature.eq(feature)),
        )
        .set(&update_feature)
        .get_result(&mut conn)
        .await
        .map_err(Error::from_std_error)
    }
}
