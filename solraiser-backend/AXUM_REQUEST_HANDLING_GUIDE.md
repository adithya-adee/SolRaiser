# Axum Request Handling Guide: Query Parameters, Path Parameters, and Request Bodies

This guide explains how to handle different types of request data in Axum using Serde for serialization/deserialization.

## Table of Contents
1. [Query Parameters](#query-parameters)
2. [Path Parameters](#path-parameters)
3. [Request Body (JSON)](#request-body-json)
4. [Combining Multiple Extractors](#combining-multiple-extractors)
5. [Best Practices](#best-practices)

---

## Query Parameters

Query parameters are key-value pairs in the URL after the `?` symbol (e.g., `/search?q=rust&limit=10`).

### Basic Usage

```rust
use axum::{extract::Query, routing::get, Router, Json};
use serde::{Deserialize, Serialize};

// Define a struct for your query parameters
#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,      // Optional parameter
    pub limit: Option<u32>,     // Optional parameter
    pub offset: Option<u32>,    // Optional parameter
}

// Handler function
async fn search(Query(params): Query<SearchQuery>) -> Json<SearchResponse> {
    // Access the query parameters
    let query = params.q.unwrap_or_else(|| "default".to_string());
    let limit = params.limit.unwrap_or(10);
    let offset = params.offset.unwrap_or(0);

    // Your logic here...
    Json(SearchResponse {
        query,
        limit,
        offset,
        results: vec![],
    })
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub query: String,
    pub limit: u32,
    pub offset: u32,
    pub results: Vec<String>,
}

// Register the route
fn app() -> Router {
    Router::new()
        .route("/search", get(search))
}
```

### Example Requests

```bash
# All parameters optional
GET /search
GET /search?q=rust
GET /search?q=rust&limit=20
GET /search?q=rust&limit=20&offset=5
```

### Key Points

- **Optional vs Required**: Use `Option<T>` for optional parameters, use `T` for required parameters
- **Type Coercion**: Axum automatically converts string query values to the appropriate type
- **Multiple Values**: Use `Vec<T>` to accept multiple values for the same parameter
- **Validation**: Perform validation after extracting the query parameters

### Advanced Example with Validation

```rust
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug)]
enum AppError {
    BadRequest(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
        };
        (status, message).into_response()
    }
}

async fn search(
    Query(params): Query<SearchQuery>
) -> Result<Json<SearchResponse>, AppError> {
    // Validate query parameters
    let limit = params.limit.unwrap_or(10);
    
    if limit > 100 {
        return Err(AppError::BadRequest(
            "Limit cannot exceed 100".to_string()
        ));
    }

    let query = params.q.unwrap_or_else(|| "default".to_string());
    
    if query.len() > 200 {
        return Err(AppError::BadRequest(
            "Query too long (max 200 characters)".to_string()
        ));
    }

    // Your logic...
    Ok(Json(SearchResponse {
        query,
        limit,
        results: vec![],
    }))
}
```

---

## Path Parameters

Path parameters are dynamic segments in the URL path (e.g., `/user/123/posts/456`).

### Basic Usage

```rust
use axum::{extract::Path, routing::get, Router};

// Single path parameter
async fn get_user(Path(user_id): Path<String>) -> String {
    format!("User ID: {}", user_id)
}

// Register the route with colon syntax
fn app() -> Router {
    Router::new()
        .route("/user/:user_id", get(get_user))
}
```

### Example Requests

```bash
GET /user/123           # user_id = "123"
GET /user/abc-def       # user_id = "abc-def"
```

### Multiple Path Parameters

```rust
// Using a tuple to extract multiple path parameters
async fn get_user_post(
    Path((user_id, post_id)): Path<(String, u64)>
) -> String {
    format!("User: {}, Post: {}", user_id, post_id)
}

fn app() -> Router {
    Router::new()
        .route("/user/:user_id/post/:post_id", get(get_user_post))
}
```

### Example Requests

```bash
GET /user/john/post/42  # user_id = "john", post_id = 42
```

### Using a Struct for Path Parameters

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct UserPostParams {
    user_id: String,
    post_id: u64,
}

async fn get_user_post(
    Path(params): Path<UserPostParams>
) -> String {
    format!("User: {}, Post: {}", params.user_id, params.post_id)
}

fn app() -> Router {
    Router::new()
        .route("/user/:user_id/post/:post_id", get(get_user_post))
}
```

### Key Points

- **Type Safety**: Axum automatically parses path parameters to the specified type
- **Order Matters**: When using tuples, the order must match the URL pattern
- **Validation**: Always validate path parameters (check for empty strings, valid ranges, etc.)

### Advanced Example with Validation

```rust
async fn get_transaction(
    Path(signature): Path<String>
) -> Result<Json<TransactionResponse>, AppError> {
    // Validate the signature
    if signature.is_empty() {
        return Err(AppError::BadRequest(
            "Transaction signature cannot be empty".to_string()
        ));
    }

    if signature.len() < 32 || signature.len() > 88 {
        return Err(AppError::BadRequest(
            "Invalid transaction signature format".to_string()
        ));
    }

    // Fetch transaction...
    Ok(Json(TransactionResponse {
        signature,
        data: serde_json::json!({}),
    }))
}
```

---

## Request Body (JSON)

Request bodies are typically used with POST, PUT, and PATCH requests to send structured data.

### Basic Usage

```rust
use axum::{extract::Json, routing::post, Router};
use serde::{Deserialize, Serialize};

// Define the request body structure
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub age: Option<u32>,
}

// Define the response structure
#[derive(Debug, Serialize)]
pub struct CreateUserResponse {
    pub user_id: String,
    pub username: String,
    pub email: String,
}

// Handler function
async fn create_user(
    Json(payload): Json<CreateUserRequest>
) -> Json<CreateUserResponse> {
    // Access the request body
    let user_id = uuid::Uuid::new_v4().to_string();

    Json(CreateUserResponse {
        user_id,
        username: payload.username,
        email: payload.email,
    })
}

fn app() -> Router {
    Router::new()
        .route("/user", post(create_user))
}
```

### Example Request

```bash
POST /user
Content-Type: application/json

{
  "username": "johndoe",
  "email": "john@example.com",
  "age": 25
}
```

### Key Points

- **`Json` Extractor**: Axum's `Json<T>` automatically deserializes the request body
- **Content-Type**: The client must send `Content-Type: application/json`
- **Validation**: Serde will fail if required fields are missing or have wrong types
- **Optional Fields**: Use `Option<T>` for optional fields

### Advanced Example with Validation

```rust
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    
    #[validate(email)]
    pub email: String,
    
    #[validate(range(min = 18, max = 120))]
    pub age: Option<u32>,
}

async fn create_user(
    Json(payload): Json<CreateUserRequest>
) -> Result<Json<CreateUserResponse>, AppError> {
    // Validate the payload
    payload.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    // Validate business logic
    if payload.username.is_empty() {
        return Err(AppError::BadRequest(
            "Username cannot be empty".to_string()
        ));
    }

    // Create user...
    let user_id = uuid::Uuid::new_v4().to_string();

    Ok(Json(CreateUserResponse {
        user_id,
        username: payload.username,
        email: payload.email,
    }))
}
```

### Nested Structures

```rust
#[derive(Debug, Deserialize)]
pub struct Address {
    pub street: String,
    pub city: String,
    pub country: String,
    pub postal_code: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub address: Address,
    pub tags: Vec<String>,
}

async fn create_user(
    Json(payload): Json<CreateUserRequest>
) -> Json<CreateUserResponse> {
    // Access nested fields
    println!("City: {}", payload.address.city);
    println!("Tags: {:?}", payload.tags);

    // Your logic...
    Json(CreateUserResponse {
        user_id: "123".to_string(),
        username: payload.username,
        email: payload.email,
    })
}
```

### Example Request

```bash
POST /user
Content-Type: application/json

{
  "username": "johndoe",
  "email": "john@example.com",
  "address": {
    "street": "123 Main St",
    "city": "New York",
    "country": "USA",
    "postal_code": "10001"
  },
  "tags": ["developer", "rust", "blockchain"]
}
```

---

## Combining Multiple Extractors

You can combine path parameters, query parameters, and request body in a single handler.

### Example: All Three Together

```rust
use axum::{
    extract::{Json, Path, Query, State},
    routing::post,
    Router,
};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
struct UserIdPath {
    user_id: String,
}

#[derive(Debug, Deserialize)]
struct PaginationQuery {
    page: Option<u32>,
    per_page: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct UpdateUserRequest {
    username: Option<String>,
    email: Option<String>,
}

#[derive(Debug, Serialize)]
struct UpdateUserResponse {
    user_id: String,
    username: String,
    email: String,
    page: u32,
}

#[derive(Clone)]
struct AppState {
    // Your shared state
}

async fn update_user(
    Path(params): Path<UserIdPath>,           // Path parameters
    Query(query): Query<PaginationQuery>,     // Query parameters
    State(_state): State<Arc<AppState>>,      // Application state
    Json(payload): Json<UpdateUserRequest>,   // Request body
) -> Result<Json<UpdateUserResponse>, AppError> {
    let page = query.page.unwrap_or(1);
    
    // Your update logic...
    Ok(Json(UpdateUserResponse {
        user_id: params.user_id,
        username: payload.username.unwrap_or("default".to_string()),
        email: payload.email.unwrap_or("default@example.com".to_string()),
        page,
    }))
}

fn app() -> Router {
    let state = Arc::new(AppState {});
    
    Router::new()
        .route("/user/:user_id", post(update_user))
        .with_state(state)
}
```

### Example Request

```bash
POST /user/123?page=2&per_page=20
Content-Type: application/json

{
  "username": "newname",
  "email": "newemail@example.com"
}
```

### Extractor Order

**Important**: The order of extractors matters! Follow this order:

1. `Path` - Path parameters
2. `Query` - Query parameters
3. `State` - Application state
4. `Json` - Request body (must be last!)

```rust
// ✅ Correct order
async fn handler(
    Path(id): Path<String>,
    Query(params): Query<SearchQuery>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<RequestBody>,
) { }

// ❌ Wrong order (will not compile)
async fn handler(
    Json(body): Json<RequestBody>,
    Path(id): Path<String>,  // Error!
) { }
```

---

## Best Practices

### 1. Use Result Types for Error Handling

```rust
async fn handler(
    Json(payload): Json<Request>
) -> Result<Json<Response>, AppError> {
    // Validate input
    if payload.field.is_empty() {
        return Err(AppError::BadRequest("Field required".to_string()));
    }

    // Process...
    Ok(Json(Response { /* ... */ }))
}
```

### 2. Create Custom Error Types

```rust
#[derive(Debug)]
enum AppError {
    BadRequest(String),
    NotFound(String),
    InternalServerError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::InternalServerError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, msg)
            }
        };

        (status, Json(serde_json::json!({
            "error": message
        }))).into_response()
    }
}
```

### 3. Validate Input

Always validate user input:

```rust
async fn create_campaign(
    Json(payload): Json<CreateCampaignRequest>
) -> Result<Json<CreateCampaignResponse>, AppError> {
    // Validate required fields
    if payload.title.trim().is_empty() {
        return Err(AppError::BadRequest("Title is required".to_string()));
    }

    // Validate ranges
    if payload.goal_amount == 0 {
        return Err(AppError::BadRequest("Goal must be greater than 0".to_string()));
    }

    // Validate string lengths
    if payload.description.len() > 1000 {
        return Err(AppError::BadRequest("Description too long".to_string()));
    }

    // Process the valid request...
    Ok(Json(CreateCampaignResponse { /* ... */ }))
}
```

### 4. Use Proper Types

```rust
// ✅ Good: Use proper types
#[derive(Deserialize)]
struct CreateCampaign {
    pub title: String,
    pub goal_amount: u64,        // Not String
    pub deadline: i64,           // Unix timestamp
    pub metadata_url: String,
}

// ❌ Bad: Everything as String
#[derive(Deserialize)]
struct CreateCampaign {
    pub title: String,
    pub goal_amount: String,     // Should be u64
    pub deadline: String,        // Should be i64
    pub metadata_url: String,
}
```

### 5. Document Your Endpoints

```rust
/// Creates a new fundraising campaign
/// 
/// # Request Body
/// - `title`: Campaign title (3-100 characters)
/// - `goal_amount`: Target amount in lamports (> 0)
/// - `deadline`: Unix timestamp (must be in future)
/// - `metadata_url`: IPFS URL for metadata (max 200 chars)
/// 
/// # Responses
/// - `200 OK`: Campaign created successfully
/// - `400 Bad Request`: Invalid input
/// - `500 Internal Server Error`: Server error
async fn create_campaign(
    Json(payload): Json<CreateCampaignRequest>
) -> Result<Json<CreateCampaignResponse>, AppError> {
    // Implementation...
}
```

### 6. Use Serde Attributes for Flexibility

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct CreateUserRequest {
    // Rename field from JSON
    #[serde(rename = "user_name")]
    pub username: String,

    // Provide default value
    #[serde(default)]
    pub is_active: bool,

    // Skip if null
    #[serde(skip_serializing_if = "Option::is_none")]
    pub middle_name: Option<String>,

    // Custom deserializer
    #[serde(deserialize_with = "deserialize_email")]
    pub email: String,
}
```

### 7. Handle CORS Properly

```rust
use tower_http::cors::{Any, CorsLayer};

let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_methods(Any)
    .allow_headers(Any);

let app = Router::new()
    .route("/api/endpoint", post(handler))
    .layer(cors);
```

---

## Complete Example

Here's a complete example combining everything:

```rust
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

// State
#[derive(Clone)]
struct AppState {
    // Database connection, config, etc.
}

// Error handling
#[derive(Debug)]
enum AppError {
    BadRequest(String),
    NotFound(String),
    InternalServerError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::InternalServerError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, msg)
            }
        };

        (status, Json(serde_json::json!({
            "error": message
        }))).into_response()
    }
}

// Request/Response types
#[derive(Debug, Deserialize)]
struct CreateCampaignRequest {
    pub title: String,
    pub description: String,
    pub goal_amount: u64,
    pub deadline: i64,
}

#[derive(Debug, Serialize)]
struct CreateCampaignResponse {
    pub campaign_id: String,
    pub title: String,
    pub created_at: i64,
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    pub q: Option<String>,
    pub limit: Option<u32>,
}

// Handlers
async fn create_campaign(
    Json(payload): Json<CreateCampaignRequest>,
) -> Result<Json<CreateCampaignResponse>, AppError> {
    // Validate
    if payload.title.trim().is_empty() {
        return Err(AppError::BadRequest("Title required".to_string()));
    }

    if payload.goal_amount == 0 {
        return Err(AppError::BadRequest("Goal must be > 0".to_string()));
    }

    // Create campaign...
    Ok(Json(CreateCampaignResponse {
        campaign_id: "camp_123".to_string(),
        title: payload.title,
        created_at: chrono::Utc::now().timestamp(),
    }))
}

async fn get_campaign(
    Path(campaign_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    if campaign_id.is_empty() {
        return Err(AppError::BadRequest("Invalid campaign ID".to_string()));
    }

    // Fetch campaign...
    Ok(Json(serde_json::json!({
        "campaign_id": campaign_id,
        "title": "Sample Campaign"
    })))
}

async fn search_campaigns(
    Query(params): Query<SearchQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let query = params.q.unwrap_or_default();
    let limit = params.limit.unwrap_or(10).min(100);

    Ok(Json(serde_json::json!({
        "query": query,
        "limit": limit,
        "results": []
    })))
}

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState {});

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/campaigns", post(create_campaign))
        .route("/campaigns/:campaign_id", get(get_campaign))
        .route("/campaigns/search", get(search_campaigns))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4000")
        .await
        .expect("Failed to bind");

    println!("Server running on http://0.0.0.0:4000");

    axum::serve(listener, app)
        .await
        .expect("Server failed");
}
```

---

## Summary

| Type | Extractor | Example | Usage |
|------|-----------|---------|-------|
| **Query Params** | `Query<T>` | `GET /search?q=rust&limit=10` | Optional parameters, filters, pagination |
| **Path Params** | `Path<T>` | `GET /user/:id` | Resource identifiers |
| **JSON Body** | `Json<T>` | `POST /user` with JSON | Creating/updating resources |
| **State** | `State<T>` | Shared app state | Database, config, etc. |

Remember:
- Always validate input
- Use proper error handling
- Keep extractors in the correct order
- Use appropriate types (not everything is a String!)
- Document your endpoints

---

**Happy coding with Axum! 🚀**
