use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct VerifyingKey(ed25519_dalek::VerifyingKey);

impl VerifyingKey {
    pub fn new(key: ed25519_dalek::VerifyingKey) -> Self {
        Self(key)
    }
}
