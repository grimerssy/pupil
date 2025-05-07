use std::str::FromStr;

use anyhow::anyhow;
use email_address::EmailAddress;

use crate::app::validation::{Validation, ValidationFailure};

const EMAIL: &str = "Email";

const MAX_LENGTH: usize = 50;

#[derive(Clone, Debug, sqlx::Type)]
#[sqlx(transparent)]
pub struct Email(String);

impl TryFrom<String> for Email {
    type Error = ValidationFailure<String>;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Validation::new(value)
            .check(|v| EmailAddress::from_str(v))
            .check_or_else(
                |v| v.len() <= MAX_LENGTH,
                || anyhow!("{EMAIL} must not exceed {MAX_LENGTH} characters"),
            )
            .finish()
            .map(Self)
    }
}

impl From<Email> for String {
    fn from(value: Email) -> Self {
        value.0
    }
}
