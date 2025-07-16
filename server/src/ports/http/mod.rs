use axum::{response::Json, routing::get, Router};
use serde_json::json;
use std::{future::IntoFuture, net::SocketAddr};
use tower_http::trace::TraceLayer;

use crate::config::Config;

pub mod trace;

async fn health() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

#[tracing::instrument()]
pub async fn start_server(config: Config) -> anyhow::Result<impl IntoFuture> {
    let address: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    let host = address.ip();
    let port = address.port();

    let app = Router::new().route("/api/health", get(health)).layer(
        TraceLayer::new_for_http()
            .make_span_with(trace::make_span_with(host, port))
            .on_response(trace::on_response())
            .on_failure(trace::on_failure()),
    );

    let listener = tokio::net::TcpListener::bind(&address).await?;
    tracing::info!("Server listening on http://{}", address);

    Ok(axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    ))
}
