use anyhow::{Context, Result};
use axum::{Router, extract::State, response::Json, routing::get, serve};
use serde_json::json;
use std::future::IntoFuture;
use tokio::net::TcpListener;

use crate::adapters::clock::Clock;
use crate::cross_cutting::config::ServerConfig;

async fn health<C: Clock>(State(state): State<ServerState<C>>) -> Json<serde_json::Value> {
    let timestamp = state.clock.now().timestamp();
    Json(json!({
        "status": "healthy",
        "timestamp": timestamp,
    }))
}

#[derive(Clone)]
struct ServerState<C> {
    pub clock: C,
}

impl<C: Clock> ServerState<C> {
    pub fn new(clock: C) -> Self {
        Self { clock }
    }
}

pub async fn start_server<C: Clock>(config: &ServerConfig, clock: C) -> Result<impl IntoFuture> {
    let state = ServerState::new(clock);
    let router = Router::new()
        .route("/api/health", get(health))
        .with_state(state);

    let address = &config.address;
    let listener = TcpListener::bind(address)
        .await
        .context(format!("Failed to bind to address {address}"))?;
    println!("Server listening on http://{address}.");
    Ok(serve(listener, router.into_make_service()))
}
