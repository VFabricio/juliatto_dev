use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use serde_json::json;
use tracing::{error, instrument};

use crate::adapters::{
    Adapter,
    token_validator::{TokenValidator, ValidateTokenError},
};
use crate::cross_cutting::error::LogError;

#[derive(Clone, Debug)]
pub struct TurnstileTokenValidator {
    client: Client,
    route: String,
    secret: SecretString,
}

impl TurnstileTokenValidator {
    pub fn new(route: String, secret: SecretString) -> Result<Self> {
        let client = Client::builder()
            .build()
            .context("Failed to build reqwest client.")?;
        Ok(Self {
            client,
            route,
            secret,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct SiteverifyResponse {
    success: bool,
    error_codes: Vec<String>,
    // TODO: add hostname validation
    #[allow(dead_code)]
    hostname: Option<String>,
}

impl Adapter for TurnstileTokenValidator {}

#[async_trait]
impl TokenValidator for TurnstileTokenValidator {
    #[instrument]
    async fn validate_token<A: AsRef<str> + std::fmt::Debug + Send>(
        &self,
        token: A,
    ) -> Result<(), ValidateTokenError> {
        let secret = self.secret.expose_secret();
        let response = self
            .client
            .post(&self.route)
            .json(&json!({
                "response": token.as_ref(),
                "secret": secret,
            }))
            .send()
            .await
            .log_error()
            .map_err(|_error| ValidateTokenError::TokenValidatorUnavailable)?;

        let response = response
            .json::<SiteverifyResponse>()
            .await
            .log_error()
            .map_err(|_error| ValidateTokenError::TokenValidatorUnavailable)?;

        if !response.success {
            error!(
                "error-codes" = response.error_codes.join(","),
                "Siteverify verification failed."
            );
            return Err(
                if response
                    .error_codes
                    .iter()
                    .any(|code| code == "invalid-input-response")
                {
                    ValidateTokenError::TokenInvalid
                } else {
                    ValidateTokenError::TokenValidatorUnavailable
                },
            );
        }

        Ok(())
    }
}
