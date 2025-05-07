use anyhow::anyhow;
use secrecy::{ExposeSecret, SecretString};
use sqlx::{
    encode::IsNull, error::BoxDynError, postgres::PgArgumentBuffer, Database, Decode, Encode,
    Postgres, Type,
};

use crate::app::validation::{Validation, ValidationFailure};

const PASSWORD: &str = "Password";

const MIN_LENGTH: usize = 8;
const MAX_LENGTH: usize = 32;

#[derive(Clone, Debug)]
pub struct Password(SecretString);

#[derive(Clone, Debug)]
pub struct PasswordHash(SecretString);

impl TryFrom<SecretString> for Password {
    type Error = ValidationFailure<SecretString>;

    fn try_from(value: SecretString) -> Result<Self, Self::Error> {
        Validation::new(value)
            .check_or_else(
                |v| v.expose_secret().len() >= MIN_LENGTH,
                || anyhow!("{PASSWORD} must be at least {MIN_LENGTH} characters long"),
            )
            .check_or_else(
                |v| v.expose_secret().len() <= MAX_LENGTH,
                || anyhow!("{PASSWORD} must not exceed {MAX_LENGTH} characters"),
            )
            .check_or_else(
                |v| v.expose_secret().chars().any(char::is_lowercase),
                || anyhow!("{PASSWORD} must contain at least one lowercase letter"),
            )
            .check_or_else(
                |v| v.expose_secret().chars().any(char::is_uppercase),
                || anyhow!("{PASSWORD} must contain at least one uppercase letter"),
            )
            .check_or_else(
                |v| v.expose_secret().chars().any(|c| c.is_ascii_digit()),
                || anyhow!("{PASSWORD} must contain at least one digit"),
            )
            .check_or_else(
                |v| v.expose_secret().chars().any(|c| c.is_ascii_punctuation()),
                || anyhow!("{PASSWORD} must contain at least one special character"),
            )
            .finish()
            .map(Self)
    }
}

impl ExposeSecret<str> for Password {
    fn expose_secret(&self) -> &str {
        self.0.expose_secret()
    }
}

impl From<Password> for SecretString {
    fn from(value: Password) -> Self {
        value.0
    }
}

impl PasswordHash {
    pub fn new(value: SecretString) -> Self {
        Self(value)
    }
}

impl Type<Postgres> for PasswordHash {
    fn type_info() -> <Postgres as Database>::TypeInfo {
        Box::<str>::type_info()
    }
}

impl Encode<'_, Postgres> for PasswordHash {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
        self.0.expose_secret().encode(buf)
    }
}

impl Decode<'_, Postgres> for PasswordHash {
    fn decode(value: <Postgres as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        Box::<str>::decode(value).map(SecretString::from).map(Self)
    }
}
