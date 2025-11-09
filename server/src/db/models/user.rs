use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::db::schema::users;

#[derive(
    Debug, Clone, PartialEq, Eq, Queryable, Selectable, Identifiable, Serialize, Deserialize,
)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserModel {
    pub id: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub image_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable, AsChangeset)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub id: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub image_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = users)]
pub struct UpdateUser {
    pub display_name: Option<Option<String>>,
    pub image_url: Option<Option<String>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserDto {
    pub id: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub image_url: Option<String>,
}

impl From<CreateUserDto> for NewUser {
    fn from(dto: CreateUserDto) -> Self {
        let now = Utc::now();
        Self {
            id: dto.id,
            email: dto.email,
            display_name: dto.display_name,
            image_url: dto.image_url,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserDto {
    pub display_name: Option<Option<String>>,
    pub image_url: Option<Option<String>>,
}

impl From<UpdateUserDto> for UpdateUser {
    fn from(dto: UpdateUserDto) -> Self {
        Self {
            display_name: dto.display_name,
            image_url: dto.image_url,
            updated_at: Utc::now(),
        }
    }
}
