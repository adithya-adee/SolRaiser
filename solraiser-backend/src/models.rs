use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

/// Represents a Solana block stored in the database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Block {
    pub slot: i64,
    pub blockhash: String,
    pub parent_slot: Option<i64>,
    pub block_time: Option<i64>,
    pub indexed_at: DateTime<Utc>,
}

/// Represents a Solana transaction stored in the database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Transaction {
    pub id: i32,
    pub signature: String,
    pub slot: i64,
    pub block_time: Option<i64>,
    pub success: bool,
    pub fee: Option<i64>,
    pub indexed_at: DateTime<Utc>,
}

/// Represents account state updates stored in the database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AccountUpdates {
    pub id: i32,
    pub pubkey: String,
    pub slot: i64,
    pub lamports: Option<i64>,
    pub owner: Option<String>,
    pub data: Option<String>,
    pub indexed_at: DateTime<Utc>,
}
