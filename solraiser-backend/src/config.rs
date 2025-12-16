use anyhow::{Context, Ok};
use serde::Deserialize;

const DEFAULT_SOLANA_RPC: &str = "https://api.mainnet-beta.solana.com";
const DEFAULT_SERVER_HOST: &str = "0.0.0.0";
const DEFAULT_SERVER_PORT: u16 = 5000;
const DEFAULT_PROGRAM_ID: &str = "62NbBCCxPfR83xtgw3AaxKGHyyDdxobrcCGzA7s7LFie";

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub solana_rpc_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub program_id: String,
    pub start_slot: Option<u64>,
}

impl Config {
    pub fn from_env() -> Result<Self, anyhow::Error> {
        dotenvy::dotenv().ok();

        Ok(Config {
            database_url: std::env::var("DATABASE_URL")?,
            solana_rpc_url: std::env::var("SOLANA_RPC_URL")
                .unwrap_or_else(|_| DEFAULT_SOLANA_RPC.to_string()),
            server_host: std::env::var("SERVER_HOST")
                .unwrap_or_else(|_| DEFAULT_SERVER_HOST.to_string()),
            server_port: std::env::var("SERVER_PORT")
                .unwrap_or_else(|_| DEFAULT_SERVER_PORT.to_string())
                .parse::<u16>()
                .context("SERVER_PORT must be a valid port number (1-65535)")?,
            program_id: std::env::var("PROGRAM_ID")
                .unwrap_or_else(|_| DEFAULT_PROGRAM_ID.to_string()),
            start_slot: std::env::var("START_SLOT")
                .ok()
                .and_then(|s| s.parse().ok()),
        })
    }
}
