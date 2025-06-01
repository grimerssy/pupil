use std::str::FromStr;

use educe::Educe;
use email_address::EmailAddress;
use serde::Serialize;

use crate::app::{
    localization::LocalizedError,
    validation::{Validation, ValidationFailure},
};

const MAX_LENGTH: usize = 50;

#[derive(Educe, Clone, Debug, Serialize, sqlx::Type)]
#[educe(Into(String))]
#[sqlx(transparent)]
pub struct Email(String);

#[derive(Educe, Clone, Debug, sqlx::Type)]
#[educe(Into(String))]
#[sqlx(transparent)]
pub struct MaybeEmail(String);

impl Email {
    pub fn new(email: String) -> Result<Self, ValidationFailure<String>> {
        Validation::new(email)
            .check_or_else(
                |v| EmailAddress::from_str(v).is_ok(),
                || LocalizedError::new("INVALID_EMAIL"),
            )
            .check_or_else(
                |v| v.len() <= MAX_LENGTH,
                || LocalizedError::new("EMAIL_TOO_LONG").with_number("max", MAX_LENGTH as f64),
            )
            .finish()
            .map(Self)
    }
}

impl MaybeEmail {
    pub fn new(email: String) -> Self {
        Self(email)
    }
}

impl TryFrom<String> for Email {
    type Error = ValidationFailure<String>;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<String> for MaybeEmail {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}
