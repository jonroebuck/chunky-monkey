use axum::{
    extract::{Json, State},
    http::StatusCode,
    routing::{get, post},
    Router,
};
use chunker_core::{count_tokens, recommend_strategy, Chunker, Config, FixedSizeChunker};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    config: Arc<Config>,
}

#[derive(Deserialize)]
struct EstimateRequest {
    text: String,
    price_per_1k_tokens: Option<f64>,
}

#[derive(Serialize)]
struct EstimateResponse {
    token_count: usize,
    cost: Option<f64>,
}

#[derive(Deserialize)]
struct RecommendRequest {
    text: String,
    model: Option<String>,
    tramway_url: Option<String>,
    max_sample_tokens: Option<usize>,
}

#[derive(Serialize)]
struct RecommendResponse {
    recommendation: String,
}

#[derive(Deserialize)]
struct ChunkRequest {
    text: String,
    chunk_size: Option<usize>,
    overlap: Option<usize>,
}

#[derive(Serialize)]
struct ChunkResponse {
    chunks: Vec<String>,
    count: usize,
}

async fn health() -> &'static str {
    "OK"
}

async fn estimate(
    Json(req): Json<EstimateRequest>,
) -> Result<Json<EstimateResponse>, (StatusCode, String)> {
    let token_count = count_tokens(&req.text)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let cost = req
        .price_per_1k_tokens
        .map(|p| (token_count as f64 / 1000.0) * p);
    Ok(Json(EstimateResponse { token_count, cost }))
}

async fn recommend(
    State(state): State<AppState>,
    Json(req): Json<RecommendRequest>,
) -> Result<Json<RecommendResponse>, (StatusCode, String)> {
    let recommendation = recommend_strategy(
        &req.text,
        req.model.as_deref(),
        req.tramway_url.as_deref(),
        req.max_sample_tokens,
        Some(state.config.as_ref()),
    )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(RecommendResponse { recommendation }))
}

async fn chunk(Json(req): Json<ChunkRequest>) -> Json<ChunkResponse> {
    let chunker = FixedSizeChunker::new(
        req.chunk_size.unwrap_or(512),
        req.overlap.unwrap_or(64),
    );
    let chunks = chunker.chunk(&req.text);
    let count = chunks.len();
    Json(ChunkResponse { chunks, count })
}

#[tokio::main]
async fn main() {
    let state = AppState {
        config: Arc::new(Config::load().expect("failed to load chunky-monkey config")),
    };
    let app = Router::new()
        .route("/health", get(health))
        .route("/estimate", post(estimate))
        .route("/optimizer/recommend", post(recommend))
        .route("/chunk", post(chunk))
        .with_state(state);

    let addr = "0.0.0.0:3000";
    println!("chunky-monkey server listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
