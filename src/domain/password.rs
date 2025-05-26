use educe::Educe;
use secrecy::{ExposeSecret, SecretString};
use sqlx::{
    encode::IsNull, error::BoxDynError, postgres::PgArgumentBuffer, Database, Decode, Encode,
    Postgres, Type,
};

use crate::app::{
    error::ErrorContext,
    validation::{Validation, ValidationFailure},
};

const MIN_LENGTH: usize = 8;
const MAX_LENGTH: usize = 32;

#[derive(Educe, Clone, Debug)]
#[educe(Into(SecretString))]
pub struct Password(SecretString);

#[derive(Educe, Clone, Debug)]
#[educe(Into(SecretString))]
pub struct PasswordHash(SecretString);

#[derive(Educe, Clone, Debug)]
#[educe(Into(SecretString))]
pub struct MaybePassword(SecretString);

impl TryFrom<SecretString> for Password {
    type Error = ValidationFailure<SecretString>;

    fn try_from(value: SecretString) -> Result<Self, Self::Error> {
        Validation::new(value)
            .check_or_else(
                |v| v.expose_secret().len() >= MIN_LENGTH,
                || ErrorContext::new("PASSWORD_TOO_SHORT").with_number("min", MIN_LENGTH as f64),
            )
            .check_or_else(
                |v| v.expose_secret().len() <= MAX_LENGTH,
                || ErrorContext::new("PASSWORD_TOO_LONG").with_number("max", MAX_LENGTH as f64),
            )
            .check_or_else(
                |v| v.expose_secret().chars().any(char::is_lowercase),
                || ErrorContext::new("PASSWORD_NO_LOWERCASE"),
            )
            .check_or_else(
                |v| v.expose_secret().chars().any(char::is_uppercase),
                || ErrorContext::new("PASSWORD_NO_UPPERCASE"),
            )
            .check_or_else(
                |v| v.expose_secret().chars().any(|c| c.is_ascii_digit()),
                || ErrorContext::new("PASSWORD_NO_DIGITS"),
            )
            .check_or_else(
                |v| v.expose_secret().chars().any(|c| c.is_ascii_punctuation()),
                || ErrorContext::new("PASSWORD_NO_SPECIAL"),
            )
            .finish()
            .map(Self)
    }
}

impl From<SecretString> for MaybePassword {
    fn from(value: SecretString) -> Self {
        Self(value)
    }
}

impl ExposeSecret<str> for Password {
    fn expose_secret(&self) -> &str {
        self.0.expose_secret()
    }
}

impl ExposeSecret<str> for PasswordHash {
    fn expose_secret(&self) -> &str {
        self.0.expose_secret()
    }
}

impl ExposeSecret<str> for MaybePassword {
    fn expose_secret(&self) -> &str {
        self.0.expose_secret()
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
