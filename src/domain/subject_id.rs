use educe::Educe;
use serde::Serialize;

use crate::app::{
    localization::LocalizedError,
    validation::{Validation, ValidationFailure},
};

const MIN_LENGTH: usize = 2;
const MAX_LENGTH: usize = 50;

#[derive(Educe, Debug, Clone, Serialize, Hash, PartialEq, Eq, sqlx::Type)]
#[educe(Into(String))]
#[sqlx(transparent)]
pub struct SubjectId(String);

impl SubjectId {
    pub fn new(id: String) -> Result<Self, ValidationFailure<String>> {
        Validation::new(id)
            .check_or_else(
                |v| v.len() >= MIN_LENGTH,
                || {
                    LocalizedError::new("SUBJECT_ID_TOO_SHORT")
                        .with_number("min", MIN_LENGTH as f64)
                },
            )
            .check_or_else(
                |v| v.len() <= MAX_LENGTH,
                || LocalizedError::new("SUBJECT_ID_TOO_LONG").with_number("max", MAX_LENGTH as f64),
            )
            .finish()
            .map(Self)
    }
}

impl TryFrom<String> for SubjectId {
    type Error = ValidationFailure<String>;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}
