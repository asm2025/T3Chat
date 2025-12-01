use rust_decimal::Decimal;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::schema::{transactions, balances};

/// Transaction model (token usage tracking)
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Transaction {
    pub id: Uuid,
    pub user_id: String,
    pub message_id: Option<Uuid>,
    pub conversation_id: Option<Uuid>,
    
    // Model info (denormalized for historical accuracy)
    pub provider: String,
    pub model: String,
    
    // Token usage
    pub input_tokens: Option<i32>,
    pub output_tokens: Option<i32>,
    pub total_tokens: i32,
    
    // Cost (calculated at time of transaction)
    pub cost_per_input_token: Option<Decimal>,
    pub cost_per_output_token: Option<Decimal>,
    pub total_cost: Option<Decimal>,
    pub currency: Option<String>,
    
    // Context
    pub transaction_type: Option<String>,  // completion, embedding, image, tts, stt
    
    // Timestamps
    pub created_at: DateTime<Utc>,
}

/// New transaction creation
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = transactions)]
pub struct NewTransaction {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation_id: Option<Uuid>,
    pub provider: String,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<i32>,
    pub total_tokens: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_per_input_token: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_per_output_token: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_cost: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_type: Option<String>,
}

/// Balance model (user token/credit balance)
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = balances)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Balance {
    pub id: Uuid,
    pub user_id: String,
    
    // Token credits
    pub token_credit_balance: Option<i64>,
    pub token_credit_consumed: Option<i64>,
    
    // Monetary credits
    pub monetary_balance: Option<Decimal>,
    pub monetary_consumed: Option<Decimal>,
    pub currency: Option<String>,
    
    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// New balance creation
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = balances)]
pub struct NewBalance {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_credit_balance: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_credit_consumed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monetary_balance: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monetary_consumed: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
}

/// Balance update
#[derive(Debug, Clone, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = balances)]
pub struct UpdateBalance {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_credit_balance: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_credit_consumed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monetary_balance: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monetary_consumed: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    pub updated_at: DateTime<Utc>,
}

