use educe::Educe;
use serde::Serialize;

#[derive(Educe, Debug, Clone, Serialize)]
#[educe(Into(String))]
pub struct AuthToken(String);

impl AuthToken {
    pub fn new(value: String) -> Self {
        Self(value)
    }
}
