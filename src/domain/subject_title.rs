use educe::Educe;
use serde::Serialize;

use crate::app::{
    localization::LocalizedError,
    validation::{Validation, ValidationFailure},
};

const MIN_LENGTH: usize = 5;
const MAX_LENGTH: usize = 100;

#[derive(Educe, Debug, Clone, PartialEq, Eq, Hash, Serialize, sqlx::Type)]
#[educe(Into(String))]
#[sqlx(transparent)]
pub struct SubjectTitle(String);

impl SubjectTitle {
    pub fn new(title: String) -> Result<Self, ValidationFailure<String>> {
        Validation::new(title)
            .check_or_else(
                |v| v.len() >= MIN_LENGTH,
                || {
                    LocalizedError::new("SUBJECT_TITLE_TOO_SHORT")
                        .with_number("min", MIN_LENGTH as f64)
                },
            )
            .check_or_else(
                |v| v.len() <= MAX_LENGTH,
                || {
                    LocalizedError::new("SUBJECT_TITLE_TOO_LONG")
                        .with_number("max", MAX_LENGTH as f64)
                },
            )
            .finish()
            .map(Self)
    }
}

impl TryFrom<String> for SubjectTitle {
    type Error = ValidationFailure<String>;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}
