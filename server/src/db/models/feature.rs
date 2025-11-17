use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::Text;
use diesel::{AsExpression, FromSqlRow};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::schema::user_features;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, AsExpression, FromSqlRow,
)]
#[diesel(sql_type = Text)]
pub enum Feature {
    WebSearch,
}

impl Feature {
    pub fn as_str(&self) -> &'static str {
        match self {
            Feature::WebSearch => "web_search",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "web_search" => Some(Feature::WebSearch),
            _ => None,
        }
    }
}

impl<DB> diesel::serialize::ToSql<Text, DB> for Feature
where
    DB: diesel::backend::Backend,
    str: diesel::serialize::ToSql<Text, DB>,
{
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, DB>,
    ) -> diesel::serialize::Result {
        self.as_str().to_sql(out)
    }
}

impl<DB> diesel::deserialize::FromSql<Text, DB> for Feature
where
    DB: diesel::backend::Backend,
    String: diesel::deserialize::FromSql<Text, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let s = String::from_sql(bytes)?;
        Feature::from_str(&s).ok_or_else(|| format!("Invalid Feature value: {}", s).into())
    }
}

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = user_features)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserFeatureModel {
    pub id: Uuid,
    pub user_id: String,
    pub feature: Feature,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = user_features)]
pub struct NewUserFeature {
    pub id: Uuid,
    pub user_id: String,
    pub feature: Feature,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = user_features)]
pub struct UpdateUserFeature {
    pub enabled: Option<bool>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserFeatureDto {
    pub user_id: String,
    pub feature: Feature,
    pub enabled: bool,
}

impl From<CreateUserFeatureDto> for NewUserFeature {
    fn from(dto: CreateUserFeatureDto) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id: dto.user_id,
            feature: dto.feature,
            enabled: dto.enabled,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserFeatureDto {
    pub enabled: bool,
}

impl From<UpdateUserFeatureDto> for UpdateUserFeature {
    fn from(dto: UpdateUserFeatureDto) -> Self {
        Self {
            enabled: Some(dto.enabled),
            updated_at: Utc::now(),
        }
    }
}

