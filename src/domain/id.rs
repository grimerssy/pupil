use educe::Educe;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use squint::{tag, Id};

#[derive(Educe, Debug, Clone, sqlx::Type)]
#[educe(Into(i64))]
#[sqlx(transparent)]
pub struct DbUserId(i64);

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserId(#[serde_as(as = "DisplayFromStr")] Id<{ tag("user") }>);

impl UserId {
    pub fn new(id: Id<{ tag("user") }>) -> Self {
        Self(id)
    }
}
