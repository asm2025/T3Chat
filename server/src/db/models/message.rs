use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::Text;
use diesel::{AsExpression, FromSqlRow};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::schema::messages;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, AsExpression, FromSqlRow)]
#[diesel(sql_type = Text)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

impl MessageRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
            MessageRole::System => "system",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "user" => Some(MessageRole::User),
            "assistant" => Some(MessageRole::Assistant),
            "system" => Some(MessageRole::System),
            _ => None,
        }
    }
}

impl<DB> diesel::serialize::ToSql<Text, DB> for MessageRole
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

impl<DB> diesel::deserialize::FromSql<Text, DB> for MessageRole
where
    DB: diesel::backend::Backend,
    String: diesel::deserialize::FromSql<Text, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let s = String::from_sql(bytes)?;
        MessageRole::from_str(&s).ok_or_else(|| format!("Invalid MessageRole value: {}", s).into())
    }
}

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = messages)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MessageModel {
    pub id: Uuid,
    pub chat_id: Uuid,
    pub role: MessageRole,
    pub content: String,
    pub metadata: Option<serde_json::Value>,
    pub parent_message_id: Option<Uuid>,
    pub sequence_number: i32,
    pub created_at: DateTime<Utc>,
    pub tokens_used: Option<i32>,
    pub model_used: Option<String>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = messages)]
pub struct NewMessage {
    pub id: Uuid,
    pub chat_id: Uuid,
    pub role: MessageRole,
    pub content: String,
    pub metadata: Option<serde_json::Value>,
    pub parent_message_id: Option<Uuid>,
    pub sequence_number: i32,
    pub created_at: DateTime<Utc>,
    pub tokens_used: Option<i32>,
    pub model_used: Option<String>,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = messages)]
pub struct UpdateMessage {
    pub tokens_used: Option<i32>,
    pub model_used: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateMessageDto {
    pub chat_id: Uuid,
    pub role: MessageRole,
    pub content: String,
    pub metadata: Option<serde_json::Value>,
    pub parent_message_id: Option<Uuid>,
    pub sequence_number: i32,
}

impl From<CreateMessageDto> for NewMessage {
    fn from(dto: CreateMessageDto) -> Self {
        Self {
            id: Uuid::new_v4(),
            chat_id: dto.chat_id,
            role: dto.role,
            content: dto.content,
            metadata: dto.metadata,
            parent_message_id: dto.parent_message_id,
            sequence_number: dto.sequence_number,
            created_at: Utc::now(),
            tokens_used: None,
            model_used: None,
        }
    }
}
