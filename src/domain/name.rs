use educe::Educe;

use crate::app::{
    error::ErrorContext,
    validation::{Validation, ValidationFailure},
};

const MIN_LENGTH: usize = 2;
const MAX_LENGTH: usize = 50;

#[derive(Educe, Clone, Debug, sqlx::Type)]
#[educe(Into(String))]
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
                || ErrorContext::new("NAME_TOO_SHORT").with_number("min", MIN_LENGTH as f64),
            )
            .check_or_else(
                |v| v.len() <= MAX_LENGTH,
                || ErrorContext::new("NAME_TOO_LONG").with_number("max", MAX_LENGTH as f64),
            )
            .check_or_else(
                |v| v.chars().all(is_char_valid),
                || ErrorContext::new("NAME_INVALID_CHARS"),
            )
            .finish()
            .map(Self)
    }
}
