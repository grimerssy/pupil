use anyhow::Context;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Algorithm, Argon2, Params, PasswordVerifier, Version,
};
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;

use crate::domain::{
    login::{VerifyPassword, VerifyPasswordError},
    password::{MaybePassword, Password, PasswordHash},
    signup::HashPassword,
};

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

impl HashPassword for Hasher {
    #[tracing::instrument(skip(self))]
    fn hash_password(&self, password: &Password) -> crate::Result<PasswordHash> {
        hash_password_with(self, password)
    }
}

impl VerifyPassword for Hasher {
    #[tracing::instrument(skip(self))]
    fn verify_password(
        &self,
        password: MaybePassword,
        password_hash: PasswordHash,
    ) -> crate::Result<(), VerifyPasswordError> {
        verify_password_with(self, password, password_hash)
    }
}

fn hash_password_with(hasher: &Hasher, password: &Password) -> crate::Result<PasswordHash> {
    hasher
        .expect_argon()
        .hash_password(
            password.expose_secret().as_bytes(),
            &SaltString::generate(&mut OsRng),
        )
        .map(|hash| PasswordHash::new(SecretString::from(hash.to_string())))
        .context("hash password")
        .map_err(crate::Error::Internal)
}

fn verify_password_with(
    hasher: &Hasher,
    password: MaybePassword,
    password_hash: PasswordHash,
) -> crate::Result<(), VerifyPasswordError> {
    let password_hash = argon2::PasswordHash::new(password_hash.expose_secret())
        .context("parse stored password hash")
        .map_err(crate::Error::Internal)?;
    hasher
        .expect_argon()
        .verify_password(password.expose_secret().as_bytes(), &password_hash)
        .map_err(|_| crate::Error::Expected(VerifyPasswordError::InvalidPassword))
}
