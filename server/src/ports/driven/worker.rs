use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, instrument};

use crate::adapters::{email_sender::EmailSender, subscription_repository::SubscriptionRepository};
use crate::commands::subscriptions::{
    SendNextVerificationEmailSuccess, send_next_verification_email,
};

#[derive(Debug)]
pub struct Worker<E, R> {
    email_sender: E,
    subscription_repository: R,
    hostname: String,
}

enum ProcessingResult {
    Processed,
    Error,
    None,
}

const SECONDS_TO_WAIT_IF_EMPTY: u64 = 5;
const SECONDS_TO_WAIT_IF_ERROR: u64 = 30;

impl<E: EmailSender, R: SubscriptionRepository> Worker<E, R> {
    pub fn new(email_sender: E, subscription_repository: R, hostname: String) -> Self {
        Self {
            email_sender,
            subscription_repository,
            hostname,
        }
    }

    pub async fn run(self) {
        loop {
            match self.process_one().await {
                ProcessingResult::Processed => continue,
                ProcessingResult::None => {
                    sleep(Duration::from_secs(SECONDS_TO_WAIT_IF_EMPTY)).await;
                    continue;
                }
                ProcessingResult::Error => {
                    sleep(Duration::from_secs(SECONDS_TO_WAIT_IF_ERROR)).await;
                    continue;
                }
            }
        }
    }

    #[instrument]
    async fn process_one(&self) -> ProcessingResult {
        match send_next_verification_email(
            &self.email_sender,
            &self.subscription_repository,
            &self.hostname,
        )
        .await
        {
            Ok(SendNextVerificationEmailSuccess::Sent) => ProcessingResult::Processed,
            Ok(SendNextVerificationEmailSuccess::NoEmailsToSendVerificationTo) => {
                ProcessingResult::None
            }
            Err(e) => {
                error!(worker_error = %e, "worker error");
                ProcessingResult::Error
            }
        }
    }
}
