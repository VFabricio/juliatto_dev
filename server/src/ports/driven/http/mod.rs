mod observability;
mod problem;
mod serve_static;

use anyhow::{Context, Result};
use axum::{
    Router,
    extract::{Request, State},
    response::Json,
    routing::{get, post},
    serve,
};
use http::StatusCode;
use serde::Deserialize;
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
use crate::adapters::{
    clock::Clock, code_generator::CodeGenerator, subscription_repository::SubscriptionRepository,
    token_validator::TokenValidator,
};
use crate::commands::subscriptions::{CreateSubscriptionError, create_subscription};
use crate::cross_cutting::{
    config::{ServerConfig, StaticFileConfig},
    error::LogError,
};

async fn handle_not_found(request: Request) -> Problem {
    let path = request.uri().path();
    ProblemBuilder::ROUTE_NOT_FOUND
        .detail(Some(format!("Route {path} was not found.")))
        .with_instance(path.into())
}

async fn handle_method_not_allowed(request: Request) -> Problem {
    let path = request.uri().path();
    let method = request.method();
    ProblemBuilder::METHOD_NOT_ALLOWED
        .detail(Some(format!(
            "Method {method} not allowed for route {path}."
        )))
        .with_instance(path.into())
        .with_extension("method".into(), json!(method.as_str()))
}

#[derive(Clone)]
struct ServerState<CL, CG, R, T> {
    pub clock: CL,
    pub code_generator: CG,
    pub subscription_repository: R,
    pub token_validator: T,
}

impl<CL: Clock, CG: CodeGenerator, R: SubscriptionRepository, T: TokenValidator>
    ServerState<CL, CG, R, T>
{
    pub fn new(
        clock: CL,
        code_generator: CG,
        subscription_repository: R,
        token_validator: T,
    ) -> Self {
        Self {
            clock,
            code_generator,
            subscription_repository,
            token_validator,
        }
    }
}

#[instrument]
async fn health<CL: Clock, CG: CodeGenerator, R: SubscriptionRepository, T: TokenValidator>(
    State(ServerState { clock, .. }): State<ServerState<CL, CG, R, T>>,
) -> Json<serde_json::Value> {
    let timestamp = clock.now().timestamp();
    Json(json!({
        "status": "healthy",
        "timestamp": timestamp,
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateSubscription {
    email: String,
    name: String,
    turnstile_token: String,
}

#[instrument]
async fn create_subscription_handler<
    CL: Clock,
    CG: CodeGenerator,
    R: SubscriptionRepository,
    T: TokenValidator,
>(
    State(ServerState {
        code_generator,
        subscription_repository,
        token_validator,
        ..
    }): State<ServerState<CL, CG, R, T>>,
    Json(CreateSubscription {
        email,
        name,
        turnstile_token,
    }): Json<CreateSubscription>,
) -> Result<StatusCode, Problem> {
    create_subscription(
        email,
        name,
        turnstile_token.clone(),
        code_generator,
        subscription_repository,
        token_validator,
    )
    .await
    .log_error()
    .map(|_| StatusCode::CREATED)
    .map_err(|error| match error {
        CreateSubscriptionError::TokenInvalid => ProblemBuilder::VERIFICATION_TOKEN_INVALID
            .detail(Some(format!("Token {} is not valid.", &turnstile_token)))
            .with_extension("token".into(), json!(turnstile_token)),
        CreateSubscriptionError::TokenValidatorUnavailable => {
            ProblemBuilder::VERIFICATION_TOKEN_VALIDATION_UNAVAILABLE.detail(None)
        }
        // TODO: handle this
        CreateSubscriptionError::SubscriptionCreationFailed => todo!(),
    })
}

#[instrument]
pub async fn start_server<
    CL: Clock,
    CG: CodeGenerator,
    R: SubscriptionRepository,
    T: TokenValidator,
>(
    config: ServerConfig,
    StaticFileConfig { path }: StaticFileConfig,
    clock: CL,
    code_generator: CG,
    subscription_repository: R,
    token_validator: T,
) -> Result<impl IntoFuture> {
    let address = config.address;
    let listener = TcpListener::bind(address)
        .await
        .context(format!("Failed to bind to address {address}"))?;
    println!("Server listening on http://{address}.");

    let state = ServerState::new(
        clock,
        code_generator,
        subscription_repository,
        token_validator,
    );

    let middleware = ServiceBuilder::new()
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(make_trace_layer(address));

    let router = Router::new()
        .route_service("/", ServeStaticService::new(path.join("home.html")))
        .route_service(
            "/static/script.js",
            ServeStaticService::new(path.join("static").join("script.js")),
        )
        .route_service(
            "/static/style.css",
            ServeStaticService::new(path.join("static").join("style.css")),
        )
        .route("/api/health", get(health))
        .route("/api/newsletter", post(create_subscription_handler))
        .fallback(handle_not_found)
        .method_not_allowed_fallback(handle_method_not_allowed)
        .with_state(state)
        .layer(middleware);

    Ok(serve(listener, router.into_make_service()))
}
