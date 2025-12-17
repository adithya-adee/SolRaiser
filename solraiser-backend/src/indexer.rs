use anyhow::{Context, Result};
use base64::Engine;
use borsh::BorshDeserialize;
use solana_client::{
    pubsub_client::PubsubClient,
    rpc_config::{
        CommitmentConfig, RpcTransactionConfig, RpcTransactionLogsConfig,
        RpcTransactionLogsFilter,
    },
};
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta,
    UiTransactionEncoding,
};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::state::AppState;

pub const CHANNEL_BUFFER_SIZE: usize = 1000;
const ANCHOR_EVENT_DISCRIMINATOR: &str = "Program data: ";

#[derive(Clone)]
pub struct SolanaIndexer {
    state: Arc<AppState>,
    program_id: Pubkey,
    ws_url: String,
}

// FIX: Added #[allow(dead_code)] to suppress unused field warning
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LogMessage {
    signature: String,
    program_id: Pubkey,
    slot: u64,
}

#[derive(Debug, Clone, BorshDeserialize)]
pub struct CampaignCreated {
    pub campaign_id: u64,
    pub creator_pubkey: Pubkey,
    pub goal_amount: u64,
    pub deadline: i64,
    pub metadata_url: String,
}

#[derive(Debug, Clone, BorshDeserialize)]
pub struct CampaignDonated {
    pub campaign_id: u64,
    pub donor_pubkey: Pubkey,
    pub amount: u64,
}

#[derive(Debug, Clone, BorshDeserialize)]
pub struct CampaignWithdrawn {
    pub campaign_id: u64,
    pub creator_pubkey: Pubkey,
    pub amount: u64,
}

#[derive(Debug, Clone)]
pub enum CampaignEvent {
    Created(CampaignCreated),
    Donated(CampaignDonated),
    Withdrawn(CampaignWithdrawn),
}

impl SolanaIndexer {
    pub fn new(state: Arc<AppState>, program_id: String, ws_url: String) -> Self {
        SolanaIndexer {
            state,
            program_id: program_id.parse().unwrap(),
            ws_url,
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
            let mut last_slot = self
                .state
                .last_indexed_slot
                .write()
                .map_err(|e| anyhow::anyhow!("Failed to acquire write lock: {}", e))?;
            if log_msg.slot > *last_slot {
                *last_slot = log_msg.slot;
            }
        }

        let signature = log_msg
            .signature
            .parse::<Signature>()
            .context("Failed to parse signature")?;

        let tx_with_meta = self
            .state
            .rpc_client
            .get_transaction_with_config(
                &signature,
                RpcTransactionConfig {
                    encoding: Some(UiTransactionEncoding::Json),
                    commitment: Some(CommitmentConfig::confirmed()),
                    max_supported_transaction_version: Some(0),
                },
            )
            .context("Failed to fetch transaction")?;

        self.store_block(log_msg.slot, &tx_with_meta).await?;

        self.store_transaction(&log_msg.signature, log_msg.slot, &tx_with_meta)
            .await?;

        if let Some(meta) = tx_with_meta.transaction.meta.as_ref() {
            let logs = match &meta.log_messages {
                solana_transaction_status::option_serializer::OptionSerializer::Some(logs) => Some(logs),
                solana_transaction_status::option_serializer::OptionSerializer::Skip => None,
                solana_transaction_status::option_serializer::OptionSerializer::None => None,
            };
            
            if let Some(logs) = logs {
                if let Some(event) = Self::parse_anchor_event(logs) {
                    self.store_campaign_event(&log_msg.signature, log_msg.slot, event)
                        .await?;
                }
            }
        }

        info!("Stored tx: {}", log_msg.signature);
        Ok(())
    }

    async fn store_block(
        &self,
        slot: u64,
        tx_with_meta: &EncodedConfirmedTransactionWithStatusMeta,
    ) -> Result<()> {
        let blockhash = match &tx_with_meta.transaction.transaction {
            solana_transaction_status::EncodedTransaction::Json(ui_tx) => match &ui_tx.message {
                solana_transaction_status::UiMessage::Raw(msg) => msg.recent_blockhash.clone(),
                solana_transaction_status::UiMessage::Parsed(msg) => msg.recent_blockhash.clone(),
            },
            _ => return Ok(()),
        };

        let block_time = tx_with_meta.block_time;
        let parent_slot = if slot > 0 { Some(slot as i64 - 1) } else { None };

        sqlx::query!(
            r#"
            INSERT INTO blocks (slot, blockhash, parent_slot, block_time)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (slot) DO NOTHING
            "#,
            slot as i64,
            blockhash,
            parent_slot,
            block_time,
        )
        .execute(&self.state.db)
        .await
        .context("Failed to insert block")?;

        Ok(())
    }

    async fn store_transaction(
        &self,
        signature: &str,
        slot: u64,
        tx_with_meta: &EncodedConfirmedTransactionWithStatusMeta,
    ) -> Result<()> {
        let block_time = tx_with_meta.block_time;

        let success = tx_with_meta
            .transaction
            .meta
            .as_ref()
            .map(|m| m.err.is_none())
            .unwrap_or(false);

        let fee = tx_with_meta
            .transaction
            .meta
            .as_ref()
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

    fn parse_anchor_event(logs: &[String]) -> Option<CampaignEvent> {
        for log in logs {
            if let Some(data_str) = log.strip_prefix(ANCHOR_EVENT_DISCRIMINATOR) {
                if let Ok(data) = base64::engine::general_purpose::STANDARD.decode(data_str.trim()) {
                    if data.len() < 8 {
                        continue;
                    }

                    let _discriminator = &data[0..8];
                    let event_data = &data[8..];

                    if let Ok(event) = CampaignCreated::try_from_slice(event_data) {
                        return Some(CampaignEvent::Created(event));
                    }

                    if let Ok(event) = CampaignDonated::try_from_slice(event_data) {
                        return Some(CampaignEvent::Donated(event));
                    }

                    if let Ok(event) = CampaignWithdrawn::try_from_slice(event_data) {
                        return Some(CampaignEvent::Withdrawn(event));
                    }
                }
            }
        }
        None
    }

    async fn store_campaign_event(
        &self,
        signature: &str,
        slot: u64,
        event: CampaignEvent,
    ) -> Result<()> {
        match event {
            CampaignEvent::Created(e) => {
                sqlx::query!(
                    r#"
                    INSERT INTO campaign_events 
                    (signature, slot, event_type, campaign_id, user_pubkey, amount, goal_amount, deadline, metadata_url)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                    "#,
                    signature,
                    slot as i64,
                    "created",
                    e.campaign_id as i64,
                    e.creator_pubkey.to_string(),
                    None::<i64>,
                    Some(e.goal_amount as i64),
                    Some(e.deadline),
                    Some(e.metadata_url),
                )
                .execute(&self.state.db)
                .await
                .context("Failed to insert CampaignCreated event")?;

                info!("Stored CampaignCreated event: campaign_id={}", e.campaign_id);
            }
            CampaignEvent::Donated(e) => {
                sqlx::query!(
                    r#"
                    INSERT INTO campaign_events 
                    (signature, slot, event_type, campaign_id, user_pubkey, amount, goal_amount, deadline, metadata_url)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                    "#,
                    signature,
                    slot as i64,
                    "donated",
                    e.campaign_id as i64,
                    e.donor_pubkey.to_string(),
                    Some(e.amount as i64),
                    None::<i64>,
                    None::<i64>,
                    None::<String>,
                )
                .execute(&self.state.db)
                .await
                .context("Failed to insert CampaignDonated event")?;

                info!(
                    "Stored CampaignDonated event: campaign_id={}, amount={}",
                    e.campaign_id, e.amount
                );
            }
            CampaignEvent::Withdrawn(e) => {
                sqlx::query!(
                    r#"
                    INSERT INTO campaign_events 
                    (signature, slot, event_type, campaign_id, user_pubkey, amount, goal_amount, deadline, metadata_url)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                    "#,
                    signature,
                    slot as i64,
                    "withdrawn",
                    e.campaign_id as i64,
                    e.creator_pubkey.to_string(),
                    Some(e.amount as i64),
                    None::<i64>,
                    None::<i64>,
                    None::<String>,
                )
                .execute(&self.state.db)
                .await
                .context("Failed to insert CampaignWithdrawn event")?;

                info!(
                    "Stored CampaignWithdrawn event: campaign_id={}, amount={}",
                    e.campaign_id, e.amount
                );
            }
        }

        Ok(())
    }
}
