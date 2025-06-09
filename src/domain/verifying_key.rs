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
        #[allow(clippy::format_collect)]
        self.0
            .to_bytes()
            .iter()
            .map(|byte| format!("{byte:02X}"))
            .collect::<String>()
            .serialize(serializer)
    }
}
