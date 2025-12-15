
use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
struct AppState {
    // Add shared state here (e.g., database connection, config, etc.)
}

#[derive(Debug)]
enum AppError {
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

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionSignatureRequest {
    pub transaction_signature: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionResponse {
    pub signature: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: u64,
}

/// Root handler - simple text response
async fn root() -> &'static str {
    "SolRaiser Backend API v1.0"
}

/// Health check endpoint with JSON response
async fn health_check() -> Result<Json<HealthResponse>, AppError> {
    Ok(Json(HealthResponse {
        status: "healthy".to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?
            .as_secs(),
    }))
}

async fn fetch_transaction(
    Json(payload): Json<TransactionSignatureRequest>,
) -> Result<Json<TransactionResponse>, AppError> {
    if payload.transaction_signature.is_empty() {
        return Err(AppError::BadRequest(
            "Transaction signature cannot be empty".to_string(),
        ));
    }

    let transaction_signature = &payload.transaction_signature;

    // Create HTTP client and prepare Solana JSON-RPC request
    let client = reqwest::Client::new();
    let rpc_url = "https://api.devnet.solana.com";

    let request_body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getTransaction",
        "params": [
            transaction_signature,
            {
                "encoding": "jsonParsed",
                "maxSupportedTransactionVersion": 0
            }
        ]
    });

    let response = client
        .post(rpc_url)
        .json(&request_body)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(AppError::InternalServerError(format!(
            "Failed to fetch transaction: {}",
            response.status()
        )));
    }

    let rpc_response: serde_json::Value = response.json().await?;

    // Extract the result from the JSON-RPC response
    let data = rpc_response["result"].clone();

    if data.is_null() {
        return Err(AppError::BadRequest(format!(
            "Transaction with signature '{}' not found or invalid response.",
            transaction_signature
        )));
    }

    Ok(Json(TransactionResponse {
        signature: payload.transaction_signature.clone(),
        data,
    }))
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

    let response = client
        .post(rpc_url)
        .json(&request_body)
        .send()
        .await?;

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

    Ok(Json(TransactionResponse {
        signature,
        data,
    }))
}


#[tokio::main]
async fn main() {
    // Arc (Atomic Reference Counting) is used to share state between threads
    let app_state = Arc::new(AppState {});

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/fetch-transaction", post(fetch_transaction))
        .route("/transaction/:signature", get(get_transaction_by_signature))
        .layer(cors)
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4000")
        .await
        .expect("Failed to bind to port 4000");

    println!("🚀 Server running on http://0.0.0.0:4000");

    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}