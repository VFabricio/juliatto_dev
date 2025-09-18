use anyhow::{Context, Result, anyhow};
use config::{Config, File};
use secrecy::SecretString;
use serde::Deserialize;
use std::collections::HashMap;
use std::env::var;
use std::net::SocketAddr;
use std::path::PathBuf;

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
pub struct DatabaseConfig {
    pub connection_string: String,
}

#[derive(Debug, Deserialize)]
pub struct EmailConfig {
    pub api_key: SecretString,
    pub from: String,
    pub reply_to: String,
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
    pub hostname: String,
}

#[derive(Debug, Deserialize)]
pub struct StaticFileConfig {
    pub path: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct PackageConfig {
    pub package_name: String,
    pub workspace_name: String,
    pub version: String,
}

#[derive(Deserialize)]
pub struct TurnstileConfig {
    pub route: String,
    pub secret: SecretString,
}

#[derive(Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub email: EmailConfig,
    pub package: PackageConfig,
    pub observability: ObservabilityConfig,
    pub server: ServerConfig,
    pub static_file: StaticFileConfig,
    pub turnstile: TurnstileConfig,
}

impl AppConfig {
    pub fn load(workspace_name: &str) -> Result<Self> {
        let environment = Environment::from_env().context("Failed to read environment.")?;
        let mut builder = Config::builder()
            .set_default(
                "package",
                HashMap::from_iter(vec![
                    ("package_name".to_owned(), env!("CARGO_PKG_NAME")),
                    ("workspace_name".to_owned(), workspace_name),
                    ("version".to_owned(), env!("CARGO_PKG_VERSION")),
                ]),
            )
            .context("Failed to set default PackageConfig.")?;
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
