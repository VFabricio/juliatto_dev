use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, PartialEq)]
pub enum Environment {
    Development,
    Production,
}

impl Environment {
    pub fn from_env() -> anyhow::Result<Self> {
        let env_var = env::var("ENVIRONMENT").map_err(|_| {
            anyhow::anyhow!(
                "ENVIRONMENT variable is required. Set it to 'development' or 'production'"
            )
        })?;

        match env_var.as_str() {
            "development" => Ok(Environment::Development),
            "production" => Ok(Environment::Production),
            other => Err(anyhow::anyhow!(
                "Invalid environment: '{}'. Must be 'development' or 'production'",
                other
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

pub fn load_config() -> anyhow::Result<Config> {
    let environment = Environment::from_env()?;

    let mut builder = config::Config::builder();

    if environment == Environment::Development {
        builder = builder.add_source(config::File::with_name("config").required(false));
    }

    builder = builder.add_source(config::Environment::default().separator("__"));

    let config = builder.build()?;
    let app_config: Config = config.try_deserialize()?;

    Ok(app_config)
}
