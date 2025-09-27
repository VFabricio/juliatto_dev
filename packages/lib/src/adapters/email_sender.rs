use async_trait::async_trait;
use thiserror::Error;

use super::Adapter;

#[derive(Debug, Error)]
pub enum EmailSenderError {
    #[error("could not send email due to an internal error")]
    Unknown,
}

#[async_trait]
pub trait EmailSender: Adapter {
    // TODO: make input types less strict
    async fn send(
        &self,
        to: Vec<String>,
        subject: String,
        idempotency_key: String,
        body: String,
    ) -> Result<(), EmailSenderError>;
}
