use anyhow::{Context, Result};

use core::cross_cutting::{config::AppConfig, observability::init_observability};
use core::ports::{
    driven::worker::Worker,
    driving::{
        postgres_subscription_repository::PostgresSubscriptionRepository,
        resend_email_sender::ResendEmailSender,
    },
};

const WORKSPACE_NAME: &str = "juliatto_dev";

#[tokio::main]
async fn main() -> Result<()> {
    let config = AppConfig::load(WORKSPACE_NAME).context("Failed to load configuration.")?;

    init_observability(config.observability, config.package)
        .context("Failed to configure observability.")?;

    let email_sender = ResendEmailSender::new(
        config.email.api_key,
        config.email.from,
        config.email.reply_to,
        config.email.mode,
    );
    let subscription_repository =
        PostgresSubscriptionRepository::new(config.database.connection_string)
            .await
            .context("Could not create subscriptions repository.")?;

    let worker = Worker::new(
        email_sender,
        subscription_repository,
        config.server.hostname,
    );

    worker.run().await;

    Ok(())
}
