use anyhow::Context;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Algorithm, Argon2, Params, Version,
};
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;

use crate::domain::{
    auth::{HashPassword, HashPasswordError},
    error::{DomainError, DomainResult},
    password::{Password, PasswordHash},
};

use super::AppContext;

#[derive(Clone, Debug, Deserialize)]
pub struct HasherConfig {
    secret: SecretString,
    #[serde(flatten)]
    params: ArgonParams,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ArgonConfig {}

#[derive(Clone, Debug, Deserialize)]
pub struct ArgonParams {
    memory_size: u32,
    iterations: u32,
    parallelism_factor: u32,
    output_length: Option<usize>,
}

#[derive(Clone)]
pub struct Hasher {
    config: HasherConfig,
}

impl Hasher {
    pub fn new(config: HasherConfig) -> anyhow::Result<Self> {
        let hasher = Self { config };
        let _check_valid = hasher.argon()?;
        Ok(hasher)
    }

    fn argon(&self) -> anyhow::Result<Argon2<'_>> {
        let HasherConfig { secret, params } = &self.config;
        let params = Params::new(
            params.memory_size,
            params.iterations,
            params.parallelism_factor,
            params.output_length,
        )
        .context("validate argon params")?;
        let hasher = Argon2::new_with_secret(
            secret.expose_secret().as_bytes(),
            Algorithm::default(),
            Version::default(),
            params,
        )
        .context("validate argon secret")?;
        Ok(hasher)
    }

    fn expect_argon(&self) -> Argon2<'_> {
        self.argon().expect("valid config")
    }
}

pub fn hash_password_with(ctx: &AppContext) -> impl HashPassword {
    |password| hash_password(&ctx.hasher, password)
}

#[tracing::instrument(skip(hasher))]
fn hash_password(
    hasher: &Hasher,
    password: Password,
) -> DomainResult<PasswordHash, HashPasswordError> {
    hasher
        .expect_argon()
        .hash_password(
            password.expose_secret().as_bytes(),
            &SaltString::generate(&mut OsRng),
        )
        .map(|hash| PasswordHash::new(SecretString::from(hash.to_string())))
        .context("hash password")
        .map_err(DomainError::Internal)
}
