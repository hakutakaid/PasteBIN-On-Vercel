use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use std::sync::Arc;
use tower::ServiceBuilder;
use vercel_runtime::{run, Error};
use vercel_runtime::axum::VercelLayer;

#[derive(Clone)]
struct AppState {
    redis_client: redis::Client,
    admin_pin: String,
}

#[derive(Deserialize)]
struct CreatePaste {
    content: String,
    filename: Option<String>,
    ttl: Option<i64>, // TTL in seconds: 0 or None = permanent, >0 = expire after N seconds
}

#[derive(Deserialize)]
struct UpdatePaste {
    id: String,
    content: String,
    ttl: Option<i64>, // TTL in seconds: 0 or None = permanent, >0 = expire after N seconds
}

#[derive(Serialize)]
struct PasteJson {
    id: String,
    content: String,
}

#[derive(Deserialize)]
struct VerifyRequest {
    pin: String,
}

#[derive(Deserialize)]
struct DeleteRequest {
    id: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let redis_url = env::var("REDIS_URL")
        .or_else(|_| env::var("KV_URL"))
        .unwrap_or_else(|_| "redis://127.0.0.1/".to_string());

    let admin_pin = env::var("ADMIN_PIN").unwrap_or_else(|_| "0000".to_string());

    let client = redis::Client::open(redis_url).expect("Invalid Redis URL");

    let shared_state = Arc::new(AppState {
        redis_client: client,
        admin_pin,
    });

    let app = Router::new()
        .route("/", get(root_handler))
        .route("/api/main", get(root_handler))
        .route("/api/verify", post(verify_pin))
        .route("/api/upload", post(create_paste))
        .route("/api/update", post(update_paste))
        .route("/api/list", get(list_pastes))
        .route("/api/delete", post(delete_paste))
        .route("/raw/{id}", get(get_paste))
        .route("/paste/{id}", get(get_paste))
        .with_state(shared_state);

    let app = ServiceBuilder::new()
        .layer(VercelLayer::new())
        .service(app);

    run(app).await
}

async fn root_handler() -> &'static str {
    "Pastebin Rust API is Running on Vercel!"
}

async fn verify_pin(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<VerifyRequest>,
) -> impl IntoResponse {
    if payload.pin == state.admin_pin {
        (StatusCode::OK, Json(json!({ "success": true }))).into_response()
    } else {
        (StatusCode::UNAUTHORIZED, Json(json!({ "success": false, "error": "Invalid PIN" }))).into_response()
    }
}

async fn create_paste(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreatePaste>,
) -> impl IntoResponse {
    // Logika ID: Jika filename ada & tidak kosong pakai itu, jika tidak generate UUID pendek
    let id = if let Some(name) = payload.filename {
        if !name.trim().is_empty() {
            name
        } else {
            uuid::Uuid::new_v4().to_string()[..8].to_string()
        }
    } else {
        uuid::Uuid::new_v4().to_string()[..8].to_string()
    };

    let mut conn = match state.redis_client.get_multiplexed_async_connection().await {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Redis Connect Error: {}", e)).into_response(),
    };

    // TTL Logic: 0 or None = permanent (no expiry), >0 = expire after N seconds
    let result: Result<(), redis::RedisError> = match payload.ttl {
        Some(ttl) if ttl > 0 => conn.set_ex(&id, &payload.content, ttl as u64).await,
        _ => conn.set(&id, &payload.content).await, // Permanent - no TTL
    };

    match result {
        Ok(_) => (),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Redis Save Error: {}", e)).into_response(),
    };

    (StatusCode::CREATED, Json(json!({ "id": id, "success": true }))).into_response()
}

async fn update_paste(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpdatePaste>,
) -> impl IntoResponse {
    let mut conn = match state.redis_client.get_multiplexed_async_connection().await {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Redis Connect Error: {}", e)).into_response(),
    };

    // TTL Logic: 0 or None = permanent (no expiry), >0 = expire after N seconds
    let result: Result<(), redis::RedisError> = match payload.ttl {
        Some(ttl) if ttl > 0 => conn.set_ex(&payload.id, &payload.content, ttl as u64).await,
        _ => conn.set(&payload.id, &payload.content).await, // Permanent - no TTL
    };

    match result {
        Ok(_) => (StatusCode::OK, Json(json!({ "success": true, "id": payload.id }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Update Failed: {}", e)).into_response(),
    }
}

async fn get_paste(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let mut conn = match state.redis_client.get_multiplexed_async_connection().await {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let content: String = match conn.get(&id).await {
        Ok(s) => s,
        Err(_) => return (StatusCode::NOT_FOUND, "Paste not found".to_string()).into_response(),
    };

    content.into_response()
}

async fn list_pastes(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let mut conn = match state.redis_client.get_multiplexed_async_connection().await {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let keys: Vec<String> = match conn.keys("*").await {
        Ok(k) => k,
        Err(_) => vec![],
    };

    let mut files: Vec<serde_json::Value> = Vec::new();
    for k in keys {
        // Get TTL for each key (-1 = no expiry/permanent, -2 = key doesn't exist)
        let ttl: i64 = conn.ttl(&k).await.unwrap_or(-2);
        files.push(json!({
            "id": k,
            "key": k,
            "ttl": ttl // -1 = permanent, >0 = seconds remaining
        }));
    }

    (StatusCode::OK, Json(json!({ "success": true, "files": files }))).into_response()
}

async fn delete_paste(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<DeleteRequest>,
) -> impl IntoResponse {
    let mut conn = match state.redis_client.get_multiplexed_async_connection().await {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let result: Result<(), redis::RedisError> = conn.del(&payload.id).await;

    match result {
        Ok(_) => (StatusCode::OK, Json(json!({ "success": true }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete failed: {}", e)).into_response(),
    }
}