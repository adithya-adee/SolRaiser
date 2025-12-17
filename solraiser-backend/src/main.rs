use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use solraiser_backend::{config::Config, state::AppState};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

#[derive(Debug, Serialize, Deserialize)]
pub struct CampaignCreateRequest {
    pub name: String,
    pub description: String,
    pub image_url: String,
    pub target_amount: u64,
    pub duration: u64,
    pub creator: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let config = Config::from_env()?;

    let db_pool = PgPoolOptions::new().connect(&config.database_url).await?;

    let start_slot = if let Some(slot) = config.start_slot{
        slot
    } else {
        let result = sqlx::query!("SELECT MAX(slot) as max_slot FROM blocks").fetch_one(&db_pool).await?;

        result.max_slot.unwrap_or(0) as u64
    };

    let app_state = AppState::new(db_pool, config.solana_rpc_url.clone(), start_slot);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/transaction/:signature", get(get_transaction_by_signature))
        .layer(cors)
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4000").await?;

    println!("🚀 Server running on http://0.0.0.0:4000");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn root() -> &'static str {
    "SolRaiser Backend API v1.0"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthRespone {
    pub status: String,
    pub timestamp: u64,
}

async fn health_check() -> Result<Json<HealthRespone>, AppError> {
    Ok(Json(HealthRespone {
        status: "healthy".to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?
            .as_secs(),
    }))
}

#[derive(Serialize, Deserialize)]
pub struct TransactionResponse {
    pub signature: String,
    pub data: serde_json::Value,
}

async fn get_transaction_by_signature(
    Path(signature): Path<String>,
) -> Result<Json<TransactionResponse>, AppError> {
    if signature.is_empty() {
        return Err(AppError::BadRequest(
            "Transaction signature cannot be empty".to_string(),
        ));
    }

    let client = reqwest::Client::new();
    let rpc_url = "https://api.devnet.solana.com";

    let request_body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getTransaction",
        "params": [
            signature,
            {
                "encoding": "jsonParsed",
                "maxSupportedTransactionVersion": 0
            }
        ]
    });

    let response = client.post(rpc_url).json(&request_body).send().await?;

    if !response.status().is_success() {
        return Err(AppError::InternalServerError(format!(
            "Failed to fetch transaction: {}",
            response.status()
        )));
    }

    let rpc_response: serde_json::Value = response.json().await?;

    let data = rpc_response["result"].clone();

    if data.is_null() {
        return Err(AppError::BadRequest(format!(
            "Transaction with signature '{}' not found or invalid response.",
            signature
        )));
    }

    Ok(Json(TransactionResponse { signature, data }))
}

/// Custom Error Handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppError {
    BadRequest(String),
    InternalServerError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::InternalServerError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(serde_json::json!({
            "error": error_message
        }));

        (status, body).into_response()
    }
}

// Convert anyhow errors to AppError
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::InternalServerError(err.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::InternalServerError(err.to_string())
    }
}
