use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use squint::{aes::Aes128, tag, Id};

pub trait Cipher {
    fn cipher(&self) -> &Aes128;
}

#[derive(Debug, Clone, sqlx::Type)]
#[sqlx(transparent)]
pub struct DbUserId(i64);

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserId(#[serde_as(as = "DisplayFromStr")] Id<{ tag("user") }>);

impl UserId {
    pub fn new(db_id: DbUserId, cipher: &Aes128) -> Self {
        Self(Id::new(db_id.0, cipher))
    }
}
