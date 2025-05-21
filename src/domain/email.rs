use std::str::FromStr;

use educe::Educe;
use email_address::EmailAddress;

use crate::app::{
    error::ErrorContext,
    validation::{Validation, ValidationFailure},
};

const MAX_LENGTH: usize = 50;

#[derive(Educe, Clone, Debug, sqlx::Type)]
#[educe(Into(String))]
#[sqlx(transparent)]
pub struct Email(String);

impl TryFrom<String> for Email {
    type Error = ValidationFailure<String>;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Validation::new(value)
            .check_or_else(
                |v| EmailAddress::from_str(v).is_ok(),
                || ErrorContext::new("INVALID_EMAIL"),
            )
            .check_or_else(
                |v| v.len() <= MAX_LENGTH,
                || ErrorContext::new("EMAIL_TOO_LONG").with_number("max", MAX_LENGTH as f64),
            )
            .finish()
            .map(Self)
    }
}
