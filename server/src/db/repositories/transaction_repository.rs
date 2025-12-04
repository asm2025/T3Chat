use anyhow::{Context, Result};
use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::db::{
    DbPool,
    models::{Balance, NewBalance, NewTransaction, Transaction, UpdateBalance},
    schema::{balances, transactions},
};

#[async_trait]
pub trait TTransactionRepository: Send + Sync {
    // Transaction methods
    async fn create(&self, new_transaction: NewTransaction) -> Result<Transaction>;
    async fn get(&self, id: Uuid) -> Result<Option<Transaction>>;
    async fn list(&self, user_id: &str, limit: i64) -> Result<Vec<Transaction>>;

    // Balance methods
    async fn create_balance(&self, new_balance: NewBalance) -> Result<Balance>;
    async fn get_balance(&self, user_id: &str) -> Result<Option<Balance>>;
    async fn update_balance(&self, user_id: &str, update: UpdateBalance) -> Result<Balance>;
}

pub struct TransactionRepository {
    pool: DbPool,
}

impl TransactionRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TTransactionRepository for TransactionRepository {
    // ==========================================
    // TRANSACTION OPERATIONS
    // ==========================================

    /// Create a new transaction
    async fn create(&self, new_transaction: NewTransaction) -> Result<Transaction> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        diesel::insert_into(transactions::table)
            .values(&new_transaction)
            .get_result(&mut conn)
            .await
            .context("Failed to create transaction")
    }

    /// Get transaction by ID
    async fn get(&self, id: Uuid) -> Result<Option<Transaction>> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        transactions::table
            .filter(transactions::id.eq(id))
            .first(&mut conn)
            .await
            .optional()
            .context("Failed to get transaction")
    }

    /// List transactions by user
    async fn list(
        &self,
        user_id: &str,
        limit: i64,
    ) -> Result<Vec<Transaction>> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        transactions::table
            .filter(transactions::user_id.eq(user_id))
            .order(transactions::created_at.desc())
            .limit(limit)
            .load(&mut conn)
            .await
            .context("Failed to list transactions")
    }

    // ==========================================
    // BALANCE OPERATIONS
    // ==========================================

    /// Create a new balance
    async fn create_balance(&self, new_balance: NewBalance) -> Result<Balance> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        diesel::insert_into(balances::table)
            .values(&new_balance)
            .get_result(&mut conn)
            .await
            .context("Failed to create balance")
    }

    /// Get balance by user ID
    async fn get_balance(&self, user_id: &str) -> Result<Option<Balance>> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        balances::table
            .filter(balances::user_id.eq(user_id))
            .first(&mut conn)
            .await
            .optional()
            .context("Failed to get balance")
    }

    /// Update balance
    async fn update_balance(&self, user_id: &str, update: UpdateBalance) -> Result<Balance> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        diesel::update(balances::table)
            .filter(balances::user_id.eq(user_id))
            .set(&update)
            .get_result(&mut conn)
            .await
            .context("Failed to update balance")
    }
}
