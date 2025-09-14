use thiserror::Error;
use tracing::instrument;

use crate::adapters::token_validator::{TokenValidator, ValidateTokenError};
use crate::cross_cutting::error::LogError;

#[derive(Debug, Error)]
pub enum CreateSubscriptionError {
    #[error("the token is invalid")]
    TokenInvalid,
    #[error("it was not possible to validate the token due to an internal error")]
    TokenValidatorUnavailable,
}

impl From<ValidateTokenError> for CreateSubscriptionError {
    fn from(value: ValidateTokenError) -> Self {
        match value {
            ValidateTokenError::TokenInvalid => Self::TokenInvalid,
            ValidateTokenError::TokenValidatorUnavailable => Self::TokenValidatorUnavailable,
        }
    }
}

#[instrument]
pub async fn create_subscription<T: TokenValidator>(
    email: String,
    name: String,
    token: String,
    token_validator: T,
) -> Result<(), CreateSubscriptionError> {
    token_validator.validate_token(&token).await.log_error()?;

    Ok(())
}
