use async_trait::async_trait;
use thiserror::Error;

use super::Adapter;

#[derive(Debug, Error)]
pub enum ValidateTokenError {
    #[error("the token is invalid")]
    TokenInvalid,
    #[error("it was not possible to validate the token due to an internal error")]
    TokenValidatorUnavailable,
}

#[async_trait]
pub(crate) trait TokenValidator: Adapter {
    async fn validate_token<A: AsRef<str> + std::fmt::Debug + Send>(
        &self,
        token: A,
    ) -> Result<(), ValidateTokenError>;
}
