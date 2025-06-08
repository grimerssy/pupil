use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};

#[serde_as]
#[derive(Debug, Clone, Serialize)]
pub struct Signature(#[serde_as(as = "DisplayFromStr")] ed25519_dalek::Signature);

impl Signature {
    pub fn new(signature: ed25519_dalek::Signature) -> Self {
        Self(signature)
    }
}
