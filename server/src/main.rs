mod config;
mod http;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = config::load_config()?;
    println!("Server starting with config: {:#?}", config);

    http::start_server(config).await?;

    Ok(())
}
