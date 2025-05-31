use educe::Educe;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use squint::{tag, Id};

#[derive(Educe, Debug, Clone, sqlx::Type)]
#[educe(Into(i64))]
#[sqlx(transparent)]
pub struct DbUserId(i64);

#[serde_as]
#[derive(Educe, Clone, Debug, Serialize, Deserialize)]
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
