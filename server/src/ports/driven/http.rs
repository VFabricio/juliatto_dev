use anyhow::{Context, Result};
use axum::{Router, response::Json, routing::get, serve};
use serde_json::json;
use std::future::IntoFuture;
use tokio::net::TcpListener;

use crate::config::ServerConfig;

async fn health() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy"
    }))
}

pub async fn start_server(config: &ServerConfig) -> Result<impl IntoFuture> {
    let address = &config.address;
    let router = Router::new().route("/api/health", get(health));
    let listener = TcpListener::bind(address)
        .await
        .context(format!("Failed to bind to address {address}"))?;
    println!("Server listening on http://{address}.");
    Ok(serve(listener, router.into_make_service()))
}
