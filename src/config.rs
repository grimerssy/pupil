use std::{path::PathBuf, str::FromStr};

use anyhow::Context;
use serde::Deserialize;
use strum::{Display, EnumString, VariantNames};

use crate::{context::AppConfig, http::HttpConfig};

#[derive(Clone, Copy, Debug, Display, EnumString, VariantNames)]
#[strum(serialize_all = "lowercase")]
enum Environment {
    Development,
    Production,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub http: HttpConfig,
    #[serde(flatten)]
    pub app: AppConfig,
}

impl Config {
    pub fn init() -> anyhow::Result<Config> {
        config::Config::builder()
            .add_source(config::File::from(config_path(environment()?)?))
            .add_source(config::Environment::default().separator("__"))
            .build()
            .context("read config")?
            .try_deserialize()
            .context("parse config")
    }
}

fn environment() -> anyhow::Result<Environment> {
    std::env::var("ENVIRONMENT")
        .context("ENVIRONMENT must be present")
        .map(|env| Environment::from_str(env.as_str()))?
        .with_context(|| format!("environment must be one of: {:?}", Environment::VARIANTS))
}

fn config_path(environment: Environment) -> anyhow::Result<PathBuf> {
    std::env::current_dir()
        .context("read current working directory")
        .map(|dir| dir.join("config").join(format!("{environment}.yaml")))
        .context("read config file")
}
