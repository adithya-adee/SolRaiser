use std::sync::{Arc, RwLock};

use solana_client::rpc_client::RpcClient;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub rpc_client: Arc<RpcClient>,
    pub last_indexed_slot: Arc<RwLock<u64>>,
}

impl AppState {
    pub fn new(db: PgPool, rpc_url: String, start_slot: u64) -> Self {
        AppState {
            db,
            rpc_client: Arc::new(RpcClient::new(rpc_url)),
            /// RwLock for multiple reads and only single write at a time
            last_indexed_slot: Arc::new(RwLock::new(start_slot)),
        }
    }
}
