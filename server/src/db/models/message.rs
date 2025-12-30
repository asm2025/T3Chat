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

// DTOs for message operations
// Note: Use chat::Message, chat::NewMessage, and chat::UpdateMessage for database operations

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateMessageDto {
    pub chat_id: Uuid,
    pub role: MessageRole,
    pub content: String,
    pub metadata: Option<serde_json::Value>,
    pub parent_message_id: Option<Uuid>,
    pub sequence_number: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateMessageDto {
    pub content: Option<String>,
    pub metadata: Option<serde_json::Value>,
}
