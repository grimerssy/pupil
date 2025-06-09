use serde::Serialize;
use uuid::Uuid;

use crate::app::{
    localization::LocalizedError,
    validation::{Validation, ValidationFailure},
};

#[derive(Clone, Debug, sqlx::Type, Serialize)]
#[sqlx(transparent)]
pub struct Key(Uuid);

impl Key {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl TryFrom<String> for Key {
    type Error = ValidationFailure<String>;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Validation::new(value)
            .check_or_else(
                |v| Uuid::try_parse(v).is_ok(),
                || LocalizedError::new("INVALID_KEY"),
            )
            .finish()
            .map(|value| Self(Uuid::try_parse(&value).unwrap()))
    }
}
