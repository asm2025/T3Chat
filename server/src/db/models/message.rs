use chrono::{DateTime, Utc};
use diesel::sql_types::Text;
use diesel::{AsExpression, FromSqlRow};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, AsExpression, FromSqlRow)]
#[diesel(sql_type = Text)]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

impl MessageRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
            MessageRole::System => "system",
            MessageRole::Tool => "tool",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "user" => Some(MessageRole::User),
            "assistant" => Some(MessageRole::Assistant),
            "system" => Some(MessageRole::System),
            "tool" => Some(MessageRole::Tool),
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

// Legacy message model - kept for API compatibility
// Note: This doesn't directly map to the database anymore
// Use conversation::Message instead for database operations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

// Legacy - not used for database inserts
#[derive(Debug, Clone, Serialize, Deserialize)]
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

// Legacy - not used for database updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMessage {
    pub content: Option<String>,
    pub metadata: Option<Option<serde_json::Value>>,
    pub tokens_used: Option<i32>,
    pub model_used: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateMessageDto {
    pub content: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

impl From<UpdateMessageDto> for UpdateMessage {
    fn from(dto: UpdateMessageDto) -> Self {
        Self {
            content: dto.content,
            metadata: dto.metadata.map(Some),
            tokens_used: None,
            model_used: None,
        }
    }
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
