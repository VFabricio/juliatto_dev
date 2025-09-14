mod adapters;
mod commands;
mod cross_cutting;
mod ports;

use anyhow::{Context, Result};

use crate::cross_cutting::{config::AppConfig, observability::init_observability};
use crate::ports::{
    driven::http::start_server,
    driving::{system_clock::SystemClock, turnstile_token_validator::TurnstileTokenValidator},
};

const WORKSPACE_NAME: &str = "juliatto_dev";

#[tokio::main]
async fn main() -> Result<()> {
    let config = AppConfig::load(WORKSPACE_NAME).context("Failed to load configuration.")?;

    init_observability(config.observability, config.package)
        .context("Failed to configure observability.")?;

    let clock = SystemClock::new();
    let token_validator =
        TurnstileTokenValidator::new(config.turnstile.route, config.turnstile.secret)
            .context("Failed to build Turnstile token validator.")?;

    let server = start_server(config.server, config.static_file, clock, token_validator).await?;
    server.await;

    Ok(())
}
