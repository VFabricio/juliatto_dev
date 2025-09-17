use thiserror::Error;
use tracing::instrument;

use crate::adapters::{
    code_generator::CodeGenerator,
    email_sender::EmailSender,
    subscription_repository::{
        CreateSubscription, SubscriptionRepository, SubscriptionRepositoryError,
    },
    token_validator::{TokenValidator, ValidateTokenError},
};
use crate::cross_cutting::error::LogError;
use crate::domain::subscription::{SubscriptionStatus, create_verification_email_body};

#[derive(Debug, Error)]
pub enum CreateSubscriptionError {
    #[error("the token is invalid")]
    TokenInvalid,
    #[error("it was not possible to validate the token due to an internal error")]
    TokenValidatorUnavailable,
    #[error("it was not possible to create the subscription due to an internal error")]
    DatabaseError,
}

impl From<ValidateTokenError> for CreateSubscriptionError {
    fn from(value: ValidateTokenError) -> Self {
        match value {
            ValidateTokenError::TokenInvalid => Self::TokenInvalid,
            ValidateTokenError::TokenValidatorUnavailable => Self::TokenValidatorUnavailable,
        }
    }
}

impl From<SubscriptionRepositoryError> for CreateSubscriptionError {
    fn from(value: SubscriptionRepositoryError) -> Self {
        match value {
            SubscriptionRepositoryError::DatabaseError => Self::DatabaseError,
        }
    }
}

#[instrument]
pub async fn create_subscription<C: CodeGenerator, R: SubscriptionRepository, T: TokenValidator>(
    email: String,
    name: String,
    token: String,
    code_generator: C,
    subscription_repository: R,
    token_validator: T,
) -> Result<(), CreateSubscriptionError> {
    token_validator.validate_token(&token).await.log_error()?;

    let verification_code = code_generator.generate();
    let unsubscription_code = code_generator.generate();

    let subscription = CreateSubscription {
        name,
        email,
        status: SubscriptionStatus::SendVerificationEmail,
        verification_code,
        unsubscription_code,
    };

    // TODO: handle case where subscription already exists
    subscription_repository
        .create(&subscription)
        .await
        .log_error()?;

    Ok(())
}

pub enum SendNextVerificationEmailSuccess {
    Sent,
    NoEmailsToSendVerificationTo,
}

#[derive(Debug, Error)]
pub enum SendNextVerificationEmailError {
    #[error("it was not possible to send the next verification email due to an internal error")]
    Unknown,
}

#[instrument]
pub async fn send_next_verification_email<E: EmailSender, R: SubscriptionRepository>(
    email_sender: &E,
    subscription_repository: &R,
    hostname: &str,
) -> Result<SendNextVerificationEmailSuccess, SendNextVerificationEmailError> {
    let subscription = subscription_repository
        .get_first_subscription_to_send_verification_email_to()
        .await
        .map_err(|_| SendNextVerificationEmailError::Unknown)
        .log_error()?;

    if let Some(s) = subscription {
        let link = format!("{}/verify_email?code={}", hostname, s.verification_code);
        let body = create_verification_email_body(&s.name, &link);

        email_sender
            // TODO: include blog name in subject
            .send(
                vec![s.email],
                "Confirm your subscription".to_owned(),
                s.id.to_string(),
                body,
            )
            .await
            .map_err(|_| SendNextVerificationEmailError::Unknown)
            .log_error()?;

        subscription_repository
            .update_subscription_status(s.id, SubscriptionStatus::WaitingVerification)
            .await
            .map_err(|_| SendNextVerificationEmailError::Unknown)
            .log_error()?;
        Ok(SendNextVerificationEmailSuccess::Sent)
    } else {
        Ok(SendNextVerificationEmailSuccess::NoEmailsToSendVerificationTo)
    }
}
