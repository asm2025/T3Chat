use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{Set, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "ai_models")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub provider: AiProvider,
    pub model_id: String,
    pub display_name: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub description: Option<String>,
    pub context_window: i32,
    pub supports_streaming: bool,
    pub supports_images: bool,
    pub supports_functions: bool,
    #[sea_orm(nullable)]
    pub cost_per_token: Option<rust_decimal::Decimal>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Text")]
pub enum AiProvider {
    #[sea_orm(string_value = "openai")]
    OpenAI,
    #[sea_orm(string_value = "anthropic")]
    Anthropic,
    #[sea_orm(string_value = "google")]
    Google,
    #[sea_orm(string_value = "deepseek")]
    DeepSeek,
    #[sea_orm(string_value = "ollama")]
    Ollama,
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

pub use Column as AiModelColumn;
pub use Entity as AiModelEntity;
pub use Model as AiModelModel;

