use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AuthToken(String);

impl AuthToken {
    pub fn new(value: String) -> Self {
        Self(value)
    }
}
