mod observability;
mod problem;
mod serve_static;

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
use tower::ServiceBuilder;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tracing::instrument;

use self::{
    observability::make_trace_layer,
    problem::{Problem, ProblemBuilder},
    serve_static::ServeStaticService,
};
use crate::adapters::clock::Clock;
use crate::cross_cutting::config::{ServerConfig, StaticFileConfig};

async fn handle_not_found(request: Request) -> Problem {
    let path = request.uri().path();
    ProblemBuilder::ROUTE_NOT_FOUND
        .detail(format!("Route {path} was not found."))
        .with_instance(path.into())
}

async fn handle_method_not_allowed(request: Request) -> Problem {
    let path = request.uri().path();
    let method = request.method();
    ProblemBuilder::METHOD_NOT_ALLOWED
        .detail(format!("Method {method} not allowed for route {path}."))
        .with_instance(path.into())
        .with_extension("method".into(), json!(method.as_str()))
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

async fn health<C: Clock>(State(state): State<ServerState<C>>) -> Json<serde_json::Value> {
    let timestamp = state.clock.now().timestamp();
    Json(json!({
        "status": "healthy",
        "timestamp": timestamp,
    }))
}

#[instrument]
pub async fn start_server<C: Clock>(
    config: ServerConfig,
    StaticFileConfig { path }: StaticFileConfig,
    clock: C,
) -> Result<impl IntoFuture> {
    let address = config.address;
    let listener = TcpListener::bind(address)
        .await
        .context(format!("Failed to bind to address {address}"))?;
    println!("Server listening on http://{address}.");

    let state = ServerState::new(clock);

    let middleware = ServiceBuilder::new()
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(make_trace_layer(address));

    let router = Router::new()
        .route("/api/health", get(health))
        .route_service("/", ServeStaticService::new(path.join("home.html")))
        .route_service(
            "/static/script.js",
            ServeStaticService::new(path.join("static").join("script.js")),
        )
        .route_service(
            "/static/style.css",
            ServeStaticService::new(path.join("static").join("style.css")),
        )
        .fallback(handle_not_found)
        .method_not_allowed_fallback(handle_method_not_allowed)
        .with_state(state)
        .layer(middleware);

    Ok(serve(listener, router.into_make_service()))
}
