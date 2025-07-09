use axum::Router;
use std::net::SocketAddr;

use crate::config::Config;

pub async fn start_server(config: Config) -> anyhow::Result<()> {
    let app = Router::new();

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    println!("Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
