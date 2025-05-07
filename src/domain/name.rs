use anyhow::anyhow;

use crate::app::validation::{Validation, ValidationFailure};

const NAME: &str = "Name";

const MIN_LENGTH: usize = 2;
const MAX_LENGTH: usize = 50;

#[derive(Clone, Debug, sqlx::Type)]
#[sqlx(transparent)]
pub struct Name(String);

impl TryFrom<String> for Name {
    type Error = ValidationFailure<String>;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let is_char_valid =
            |c: char| c.is_alphabetic() || c.is_whitespace() || c == '-' || c == '\'';
        Validation::new(value)
            .check_or_else(
                |v| v.len() >= MIN_LENGTH,
                || anyhow!("{NAME} must be at least {MIN_LENGTH} characters long"),
            )
            .check_or_else(
                |v| v.len() <= MAX_LENGTH,
                || anyhow!("{NAME} must not exceed {MAX_LENGTH} characters"),
            )
            .check_or_else(
                |v| v.chars().all(is_char_valid),
                || anyhow!("{NAME} may only contain letters, hyphens, apostrophes and whitespace"),
            )
            .finish()
            .map(Self)
    }
}

impl From<Name> for String {
    fn from(value: Name) -> Self {
        value.0
    }
}
