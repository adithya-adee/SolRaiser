use anyhow::{Context, Result};
use solana_client::{
    pubsub_client::PubsubClient,
    rpc_config::{
        CommitmentConfig, RpcTransactionConfig, RpcTransactionLogsConfig,
        RpcTransactionLogsFilter,
    },
};
use solana_sdk::{
    pubkey::Pubkey,
    signature::Signature,
};
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta,
    UiTransactionEncoding,
};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::state::AppState;

pub const CHANNEL_BUFFER_SIZE: usize = 1000;

#[derive(Clone)]
pub struct SolanaIndexer {
    state: Arc<AppState>,
    program_id: Pubkey,
    ws_url: String,
}

/// Represents a log message received from Solana WebSocket
/// This is the data structure we pass between WebSocket listeners and the processing task
#[derive(Debug, Clone)]
pub struct LogMessage {
    signature: String,
    program_id: Pubkey,
    slot: u64,
}

impl SolanaIndexer {
    pub fn new(state: Arc<AppState>, program_id: String, ws_url: String) -> Self {
        SolanaIndexer {
            state: state,
            program_id: program_id.parse().unwrap(),
            ws_url: ws_url,
        }
    }

    pub async fn start(self: SolanaIndexer) -> Result<()> {
        let (tx, mut rx) = mpsc::channel::<LogMessage>(CHANNEL_BUFFER_SIZE);
        let tx_clone = tx.clone();
        let ws_url_clone = self.ws_url.clone();
        let state_clone = self.state.clone();
        let self_clone = self.clone();

        tokio::spawn(async move {
            if let Err(e) = Self::subscribe_to_program_logs(
                ws_url_clone,
                self.program_id,
                tx_clone,
                state_clone,
            )
            .await
            {
                error!("Subscription error: {:?}", e);
            }
        });

        drop(tx);

        tokio::spawn(async move {
            while let Some(log_msg) = rx.recv().await {
                if let Err(e) = self_clone.process_log_message(log_msg).await {
                    error!("Error processing log message: {:?}", e);
                }
            }
        });

        Ok(())
    }

    pub async fn subscribe_to_program_logs(
        ws_url: String,
        program_id: Pubkey,
        tx: mpsc::Sender<LogMessage>,
        _state: Arc<AppState>,
    ) -> Result<()> {
        loop {
            match Self::run_subscription(ws_url.clone(), program_id, tx.clone()).await {
                std::result::Result::Ok(_) => {
                    warn!("Subscription ended unexpectedly, reconnecting...");
                }
                std::result::Result::Err(e) => {
                    error!("Subscription error: {:?}, reconnecting in 5s...", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        }
    }

    async fn run_subscription(
        ws_url: String,
        program_id: Pubkey,
        tx: mpsc::Sender<LogMessage>,
    ) -> Result<()> {
        let program_id_str = program_id.to_string();

        let subscription = tokio::task::spawn_blocking(move || {
            let config = RpcTransactionLogsConfig {
                commitment: Some(CommitmentConfig::confirmed()),
            };

            let filter = RpcTransactionLogsFilter::Mentions(vec![program_id_str]);

            PubsubClient::logs_subscribe(&ws_url, filter, config)
        })
        .await
        .context("Failed to spawn subscription task")??;

        loop {
            match subscription.1.recv() {
                std::result::Result::Ok(response) => {
                    let signature = response.value.signature;
                    
                    let log_msg = LogMessage {
                        signature: signature.clone(),
                        program_id,
                        slot: response.context.slot,
                    };
                    
                    if tx.send(log_msg).await.is_err() {
                        warn!("Failed to send log message to processing channel");
                        break;
                    }
                }
                std::result::Result::Err(e) => {
                    error!("Error receiving log: {:?}", e);
                    break;
                }
            }
        }

        std::result::Result::Ok(())
    }
    
    async fn process_log_message(&self, log_msg: LogMessage) -> Result<()> {
        info!("Processing tx: {} (slot: {})", log_msg.signature, log_msg.slot);

        {
            let mut last_slot = self.state.last_indexed_slot.write()
                .map_err(|e| anyhow::anyhow!("Failed to acquire write lock: {}", e))?;
            if log_msg.slot > *last_slot {
                *last_slot = log_msg.slot;
            }
        }

        let signature = log_msg.signature.parse::<Signature>()
            .context("Failed to parse signature")?;
        
        let tx_with_meta = self.state.rpc_client
            .get_transaction_with_config(
                &signature,
                RpcTransactionConfig {
                    encoding: Some(UiTransactionEncoding::Json),
                    commitment: Some(CommitmentConfig::confirmed()),
                    max_supported_transaction_version: Some(0),
                },
            )
            .context("Failed to fetch transaction")?;

        self.store_transaction(
            &log_msg.signature,
            log_msg.slot,
            &tx_with_meta,
        ).await?;

        info!("Stored tx: {}", log_msg.signature);
        Ok(())
    }

    async fn store_transaction(
        &self,
        signature: &str,
        slot: u64,
        tx_with_meta: &EncodedConfirmedTransactionWithStatusMeta,
    ) -> Result<()> {
        let block_time = tx_with_meta.block_time;

        let success = tx_with_meta.transaction.meta.as_ref()
            .map(|m| m.err.is_none())
            .unwrap_or(false);

        let fee = tx_with_meta.transaction.meta.as_ref()
            .and_then(|m| Some(m.fee as i64));

        sqlx::query!(
            r#"
            INSERT INTO transactions (signature, slot, block_time, success, fee)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (signature) DO UPDATE
            SET slot = EXCLUDED.slot,
                block_time = EXCLUDED.block_time,
                success = EXCLUDED.success,
                fee = EXCLUDED.fee
            "#,
            signature,
            slot as i64,
            block_time,
            success,
            fee,
        )
        .execute(&self.state.db)
        .await
        .context("Failed to insert transaction")?;

        Ok(())
    }
}
