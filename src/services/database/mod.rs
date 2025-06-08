use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};

pub mod auth;
pub mod grades;
pub mod keys;
pub mod performance;

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

fn sql_error<E>(error: sqlx::Error) -> crate::Error<E> {
    crate::Error::internal(anyhow::Error::from(error).context("execute sql query"))
}
