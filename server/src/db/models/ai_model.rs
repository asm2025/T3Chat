use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::Text;
use diesel::{AsExpression, FromSqlRow};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::schema::ai_models;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, AsExpression, FromSqlRow,
)]
#[diesel(sql_type = Text)]
pub enum AiProvider {
    OpenAI,
    Anthropic,
    Google,
    DeepSeek,
    Ollama,
}

impl AiProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            AiProvider::OpenAI => "openai",
            AiProvider::Anthropic => "anthropic",
            AiProvider::Google => "google",
            AiProvider::DeepSeek => "deepseek",
            AiProvider::Ollama => "ollama",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "openai" => Some(AiProvider::OpenAI),
            "anthropic" => Some(AiProvider::Anthropic),
            "google" => Some(AiProvider::Google),
            "deepseek" => Some(AiProvider::DeepSeek),
            "ollama" => Some(AiProvider::Ollama),
            _ => None,
        }
    }
}

impl<DB> diesel::serialize::ToSql<Text, DB> for AiProvider
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

impl<DB> diesel::deserialize::FromSql<Text, DB> for AiProvider
where
    DB: diesel::backend::Backend,
    String: diesel::deserialize::FromSql<Text, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let s = String::from_sql(bytes)?;
        AiProvider::from_str(&s).ok_or_else(|| format!("Invalid AiProvider value: {}", s).into())
    }
}

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = ai_models)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AiModelModel {
    pub id: Uuid,
    pub provider: AiProvider,
    pub model_id: String,
    pub display_name: String,
    pub description: Option<String>,
    pub context_window: i32,
    pub supports_streaming: bool,
    pub supports_images: bool,
    pub supports_functions: bool,
    pub cost_per_token: Option<rust_decimal::Decimal>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
