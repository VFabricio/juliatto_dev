use anyhow::{Context, Result};

use core::cross_cutting::{config::AppConfig, observability::init_observability};
use core::ports::{
    driven::http::start_server,
    driving::{
        postgres_subscription_repository::PostgresSubscriptionRepository,
        random_code_generator::RandomCodeGenerator, system_clock::SystemClock,
        turnstile_token_validator::TurnstileTokenValidator,
    },
};

const WORKSPACE_NAME: &str = "juliatto_dev";

#[tokio::main]
async fn main() -> Result<()> {
    let config = AppConfig::load(WORKSPACE_NAME).context("Failed to load configuration.")?;

    init_observability(config.observability, config.package)
        .context("Failed to configure observability.")?;

    let code_generator = RandomCodeGenerator;
    let clock = SystemClock;
    let subscription_repository =
        PostgresSubscriptionRepository::new(config.database.connection_string)
            .await
            .context("Could not create subscriptions repository.")?;
    let token_validator =
        TurnstileTokenValidator::new(config.turnstile.route, config.turnstile.secret)
            .context("Failed to build Turnstile token validator.")?;

    let server = start_server(
        config.server,
        config.static_file,
        clock,
        code_generator,
        subscription_repository,
        token_validator,
    )
    .await?;

    server.await;

    Ok(())
}
