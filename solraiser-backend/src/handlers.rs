use axum::{
    extract::{Path, Query},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{models::Transaction, state::AppState};

pub async fn get_indexer_status(
    state: Arc<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let last_slot = *state
        .last_indexed_slot
        .read()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let latest_slot = state
        .rpc_client
        .get_slot()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "last_indexed_slot": last_slot,
        "latest_blockchain_slot": latest_slot,
        "slots_behind": latest_slot.saturating_sub(last_slot),
        "mode": "websocket-program-scoped"
    })))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PaginationParams {
    pub limit: i64,
    pub offset: i64,
}

pub async fn get_recent_blocks(
    state: Arc<AppState>,
    Query(query): Query<PaginationParams>,
) -> Result<Json<Vec<Transaction>>, StatusCode> {
    let transactions = sqlx::query_as::<_, Transaction>(
        r#"
        SELECT id, signature, slot, block_time, success, fee, indexed_at
        FROM transactions ORDER BY slot DESC, id DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(query.limit)
    .bind(query.offset)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if transactions.is_empty() {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(Json(transactions))
}

pub async fn get_transaction_by_signature(
    state: Arc<AppState>,
    Path(signature): Path<String>,
) -> Result<Json<Vec<Transaction>>, StatusCode> {
    let transactions = sqlx::query_as::<_, Transaction>(
        r#"
        SELECT id, signature, slot, block_time, success, fee, indexed_at
        FROM transactions
        WHERE signature = $1"#,
    )
    .bind(signature)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if transactions.is_empty() {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(Json(transactions))
}
