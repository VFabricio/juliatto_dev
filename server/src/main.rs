mod adapters;
mod cross_cutting;
mod ports;

use anyhow::{Context, Result};

use crate::cross_cutting::config::AppConfig;
use crate::ports::{driven::http::start_server, driving::system_clock::SystemClock};

#[tokio::main]
async fn main() -> Result<()> {
    let config = AppConfig::load().context("Failed to load configuration.")?;
    let clock = SystemClock::new();
    let server = start_server(&config.server, clock).await?;
    server.await;
    Ok(())
}
