use anyhow::Context;
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};

use crate::{context::AppContext, error::InternalError};

#[derive(Clone, Debug, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: SecretString,
    pub database: String,
    pub require_ssl: bool,
}

#[derive(Clone)]
pub struct Database {
    pool: sqlx::PgPool,
}

impl Database {
    pub fn new(config: DatabaseConfig) -> Self {
        let ssl_mode = if config.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };
        let connection_options = PgConnectOptions::new()
            .host(&config.host)
            .port(config.port)
            .username(&config.user)
            .password(config.password.expose_secret())
            .database(&config.database)
            .ssl_mode(ssl_mode);
        let pool = PgPoolOptions::new().connect_lazy_with(connection_options);
        Self { pool }
    }
}

#[tracing::instrument(skip(ctx), ret, err(Debug))]
pub async fn fetch_name(ctx: AppContext, name: &str) -> Result<String, InternalError> {
    let (name,) = sqlx::query_as("select $1;")
        .bind(name)
        .fetch_one(&ctx.database.pool)
        .await
        .context("execute query")?;
    Ok(name)
}
