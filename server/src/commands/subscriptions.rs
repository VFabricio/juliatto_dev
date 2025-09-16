use thiserror::Error;
use tracing::instrument;

use crate::adapters::{
    code_generator::CodeGenerator,
    subscription_repository::{SubscriptionRepository, SubscriptionRepositoryError},
    token_validator::{TokenValidator, ValidateTokenError},
};
use crate::cross_cutting::error::LogError;
use crate::domain::subscription::{Subscription, SubscriptionStatus};

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

    let subscription = Subscription {
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
