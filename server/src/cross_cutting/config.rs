use anyhow::{Context, Result, anyhow};
use config::{Config, File};
use serde::Deserialize;
use std::env::var;
use std::net::SocketAddr;

#[derive(Eq, PartialEq)]
pub enum Environment {
    Development,
    Production,
}

const CONFIG_FILE: &str = "config";
const ENVIRONMENT_VARIABLE: &str = "ENV";
const ENVIRONMENT_SEPARATOR: &str = "__";

impl Environment {
    fn from_env() -> Result<Self> {
        let env_var = var(ENVIRONMENT_VARIABLE).context(format!(
            "{ENVIRONMENT_VARIABLE} variable is required. Set it to  'dev' or 'prod'.",
        ))?;

        match env_var.as_ref() {
            "dev" => Ok(Environment::Development),
            "prod" => Ok(Environment::Production),
            other => Err(anyhow!(
                "Invalid environment {other}. Must be either 'dev' or 'prod'.",
            )),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ObservabilityLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Deserialize)]
pub struct ObservabilityConfig {
    pub endpoint: String,
    pub level: ObservabilityLevel,
    pub service_name: String,
    pub service_version: String,
}

impl AsRef<str> for ObservabilityLevel {
    fn as_ref(&self) -> &str {
        match self {
            ObservabilityLevel::Trace => "trace",
            ObservabilityLevel::Debug => "debug",
            ObservabilityLevel::Info => "info",
            ObservabilityLevel::Warn => "warn",
            ObservabilityLevel::Error => "error",
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub address: SocketAddr,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub observability: ObservabilityConfig,
    pub server: ServerConfig,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let environment = Environment::from_env().context("Failed to read environment.")?;
        let mut builder = Config::builder();
        if environment == Environment::Development {
            builder = builder.add_source(File::with_name(CONFIG_FILE).required(false));
        }
        builder =
            builder.add_source(config::Environment::default().separator(ENVIRONMENT_SEPARATOR));
        let config = builder
            .build()
            .context("Failed to create configuration builder.")?;

        config
            .try_deserialize()
            .context("Failed to deserialize configuration.")
    }
}
