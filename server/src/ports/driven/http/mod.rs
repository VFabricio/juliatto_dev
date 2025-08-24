mod observability;
mod problem;

use anyhow::{Context, Result};
use axum::{
    Router,
    extract::{Request, State},
    response::Json,
    routing::get,
    serve,
};
use serde_json::json;
use std::future::IntoFuture;
use tokio::net::TcpListener;
use tracing::instrument;

use self::{
    observability::make_trace_layer,
    problem::{Problem, ProblemBuilder},
};
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

async fn handle_not_found(request: Request) -> Problem {
    let path = request.uri().path();
    ProblemBuilder::ROUTE_NOT_FOUND
        .detail(format!("Route {path} was not found."))
        .with_instance(path.into())
}

#[instrument]
pub async fn start_server<C: Clock>(config: ServerConfig, clock: C) -> Result<impl IntoFuture> {
    let address = config.address;
    let listener = TcpListener::bind(address)
        .await
        .context(format!("Failed to bind to address {address}"))?;
    println!("Server listening on http://{address}.");

    let state = ServerState::new(clock);
    let router = Router::new()
        .route("/api/health", get(health))
        .layer(make_trace_layer(address))
        .fallback(handle_not_found)
        .with_state(state);

    Ok(serve(listener, router.into_make_service()))
}
