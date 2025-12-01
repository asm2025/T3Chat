use anyhow::{Context, Result};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::db::{
    models::{Transaction, NewTransaction, Balance, NewBalance, UpdateBalance},
    schema::{transactions, balances},
    DbPool,
};

pub struct TransactionRepository {
    pool: DbPool,
}

impl TransactionRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    // ==========================================
    // TRANSACTION OPERATIONS
    // ==========================================

    /// Create a new transaction
    pub async fn create_transaction(&self, new_transaction: NewTransaction) -> Result<Transaction> {
        let mut conn = self.pool.get().await.context("Failed to get DB connection")?;
        
        diesel::insert_into(transactions::table)
            .values(&new_transaction)
            .get_result(&mut conn)
            .await
            .context("Failed to create transaction")
    }

    /// Get transaction by ID
    pub async fn get_transaction_by_id(&self, id: Uuid) -> Result<Option<Transaction>> {
        let mut conn = self.pool.get().await.context("Failed to get DB connection")?;
        
        transactions::table
            .filter(transactions::id.eq(id))
            .first(&mut conn)
            .await
            .optional()
            .context("Failed to get transaction")
    }

    /// List transactions by user
    pub async fn list_transactions_by_user(&self, user_id: &str, limit: i64) -> Result<Vec<Transaction>> {
        let mut conn = self.pool.get().await.context("Failed to get DB connection")?;
        
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
    pub async fn create_balance(&self, new_balance: NewBalance) -> Result<Balance> {
        let mut conn = self.pool.get().await.context("Failed to get DB connection")?;
        
        diesel::insert_into(balances::table)
            .values(&new_balance)
            .get_result(&mut conn)
            .await
            .context("Failed to create balance")
    }

    /// Get balance by user ID
    pub async fn get_balance_by_user(&self, user_id: &str) -> Result<Option<Balance>> {
        let mut conn = self.pool.get().await.context("Failed to get DB connection")?;
        
        balances::table
            .filter(balances::user_id.eq(user_id))
            .first(&mut conn)
            .await
            .optional()
            .context("Failed to get balance")
    }

    /// Update balance
    pub async fn update_balance(&self, user_id: &str, update: UpdateBalance) -> Result<Balance> {
        let mut conn = self.pool.get().await.context("Failed to get DB connection")?;
        
        diesel::update(balances::table)
            .filter(balances::user_id.eq(user_id))
            .set(&update)
            .get_result(&mut conn)
            .await
            .context("Failed to update balance")
    }
}

