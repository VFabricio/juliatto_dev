use async_trait::async_trait;
use resend_rs::{Resend, types::CreateEmailBaseOptions};
use secrecy::{ExposeSecret, SecretString};
use tracing::{error, instrument};

use crate::{
    adapters::{
        Adapter,
        email_sender::{EmailSender, EmailSenderError},
    },
    cross_cutting::error::LogError,
};

#[derive(Clone, Debug)]
pub struct ResendEmailSender {
    client: Resend,
    from: String,
    reply_to: String,
}

impl ResendEmailSender {
    pub fn new(api_key: SecretString, from: String, reply_to: String) -> Self {
        Self {
            client: Resend::new(api_key.expose_secret()),
            from,
            reply_to,
        }
    }
}

impl Adapter for ResendEmailSender {}

#[async_trait]
impl EmailSender for ResendEmailSender {
    #[instrument]
    async fn send(
        &self,
        _to: Vec<String>,
        subject: String,
        idempotency_key: String,
        body: String,
    ) -> Result<(), EmailSenderError> {
        let email = CreateEmailBaseOptions::new(
            &self.from,
            vec!["delivered+verification@resend.dev"],
            subject,
        )
        .with_reply(&self.reply_to)
        .with_html(&body)
        .with_idempotency_key(&idempotency_key);
        self.client
            .emails
            .send(email)
            .await
            .map_err(|e| {
                error!(resend_error = %e, "error sending email");
                EmailSenderError::Unknown
            })
            .map(|_| ())
            .log_error()
    }
}
