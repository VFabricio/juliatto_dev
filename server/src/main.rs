mod adapters;
mod cross_cutting;
mod ports;

use anyhow::{Context, Result};

use crate::cross_cutting::{config::AppConfig, observability::init_observability};
use crate::ports::{driven::http::start_server, driving::system_clock::SystemClock};

const WORKSPACE_NAME: &str = "juliatto_dev";

#[tokio::main]
async fn main() -> Result<()> {
    let config = AppConfig::load(WORKSPACE_NAME).context("Failed to load configuration.")?;
    init_observability(config.observability, config.package)
        .context("Failed to configure observability.")?;
    let clock = SystemClock::new();
    let server = start_server(config.server, config.static_file, clock).await?;
    server.await;
    Ok(())
}
