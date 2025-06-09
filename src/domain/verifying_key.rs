use serde::Serialize;

#[derive(Debug, Clone)]
pub struct VerifyingKey(ed25519_dalek::VerifyingKey);

impl VerifyingKey {
    pub fn new(key: ed25519_dalek::VerifyingKey) -> Self {
        Self(key)
    }
}

impl Serialize for VerifyingKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        hex::serialize_upper(self.0, serializer)
    }
}
