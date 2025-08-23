mod cross_cutting;
mod ports;

use anyhow::{Context, Result};

use crate::cross_cutting::config::AppConfig;
use crate::ports::driven::http::start_server;

#[tokio::main]
async fn main() -> Result<()> {
    let config = AppConfig::load().context("Failed to load configuration.")?;
    let server = start_server(&config.server).await?;
    server.await;
    Ok(())
}
