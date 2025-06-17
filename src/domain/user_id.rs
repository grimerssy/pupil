use std::str::FromStr;

use educe::Educe;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use squint::{tag, Id};

use crate::app::{
    localization::LocalizedError,
    validation::{Validation, ValidationFailure},
};

#[derive(Educe, Debug, Clone, Copy, PartialEq, Eq, Hash, sqlx::Type)]
#[educe(Into(i64))]
#[sqlx(transparent)]
pub struct DbUserId(i64);

#[serde_as]
#[derive(Educe, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[educe(Into(Id<{tag("user")}>))]
pub struct UserId(#[serde_as(as = "DisplayFromStr")] Id<{ tag("user") }>);

impl DbUserId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }
}

impl UserId {
    pub fn new(id: Id<{ tag("user") }>) -> Self {
        Self(id)
    }
}

impl TryFrom<String> for UserId {
    type Error = ValidationFailure<String>;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Validation::new(value)
            .check_or_else(
                |v| Id::from_str(v).map(UserId::new).is_ok(),
                || LocalizedError::new("INVALID_USER_ID"),
            )
            .finish()
            .map(|v| Id::from_str(&v).map(UserId::new).unwrap())
    }
}

impl std::hash::Hash for UserId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_string().hash(state)
    }
}
