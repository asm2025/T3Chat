use async_trait::async_trait;
use chrono::{DateTime, Utc};
use emixdb::schema::Merge;
use sea_orm::{NotSet, Set, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    #[sea_orm(unique, nullable)]
    pub email: Option<String>,
    #[sea_orm(nullable)]
    pub display_name: Option<String>,
    #[sea_orm(nullable)]
    pub image_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        let now = Utc::now();
        Self {
            created_at: Set(now),
            updated_at: Set(now),
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
pub struct CreateUserDto {
    pub id: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub image_url: Option<String>,
}

impl From<CreateUserDto> for Model {
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

impl From<CreateUserDto> for ActiveModel {
    fn from(dto: CreateUserDto) -> Self {
        Self {
            id: Set(dto.id),
            email: Set(dto.email),
            display_name: Set(dto.display_name),
            image_url: Set(dto.image_url),
            created_at: NotSet,
            updated_at: NotSet,
        }
    }
}

impl Merge<ActiveModel> for CreateUserDto {
    fn merge(&self, model: &mut ActiveModel) -> bool {
        let mut changed = false;

        if let Set(ref current_email) = model.email {
            if current_email != &self.email {
                model.email = Set(self.email.clone());
                changed = true;
            }
        } else {
            model.email = Set(self.email.clone());
            changed = true;
        }

        if let Set(ref current_display_name) = model.display_name {
            if current_display_name != &self.display_name {
                model.display_name = Set(self.display_name.clone());
                changed = true;
            }
        } else {
            model.display_name = Set(self.display_name.clone());
            changed = true;
        }

        if let Set(ref current_image_url) = model.image_url {
            if current_image_url != &self.image_url {
                model.image_url = Set(self.image_url.clone());
                changed = true;
            }
        } else {
            model.image_url = Set(self.image_url.clone());
            changed = true;
        }

        changed
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserDto {
    pub display_name: Option<String>,
    pub image_url: Option<String>,
}

impl Merge<ActiveModel> for UpdateUserDto {
    fn merge(&self, model: &mut ActiveModel) -> bool {
        let mut changed = false;

        if let Some(display_name) = &self.display_name {
            model.display_name = Set(Some(display_name.clone()));
            changed = true;
        }

        if let Some(image_url) = &self.image_url {
            model.image_url = Set(Some(image_url.clone()));
            changed = true;
        }

        changed
    }
}

pub use ActiveModel as UserModelDto;
pub use Column as UserColumn;
pub use Entity as UserEntity;
pub use Model as UserModel;
