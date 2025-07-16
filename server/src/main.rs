mod config;
mod observability;
mod ports;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = config::load_config()?;

    observability::init_observability(&config.observability, "juliatto-dev-server")?;

    let server = ports::http::start_server(config).await?;

    server.await;

    Ok(())
}
