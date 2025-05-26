use std::time::Duration;

use anyhow::Context;
use jsonwebtoken::{get_current_timestamp, EncodingKey, Header};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationSeconds};

use crate::domain::{
    auth::{IssueToken, IssueTokenError},
    error::{DomainError, DomainResult},
    id::UserId,
    token::AuthToken,
};

#[serde_as]
#[derive(Clone, Debug, Deserialize)]
pub struct JwtConfig {
    #[serde_as(as = "DurationSeconds<u64>")]
    ttl: Duration,
    secret: SecretString,
}

#[derive(Clone)]
pub struct TokenIssuer {
    config: JwtConfig,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenClaims {
    iat: u64,
    exp: u64,
    user_id: UserId,
}

impl TokenIssuer {
    pub fn new(config: JwtConfig) -> Self {
        Self { config }
    }
}

impl IssueToken for TokenIssuer {
    fn issue_token(&self, user_id: UserId) -> DomainResult<AuthToken, IssueTokenError> {
        let now = get_current_timestamp();
        let claims = TokenClaims {
            iat: now,
            exp: now + self.config.ttl.as_secs(),
            user_id,
        };
        jsonwebtoken::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.secret.expose_secret().as_bytes()),
        )
        .map(AuthToken::new)
        .context("encode jwt")
        .map_err(DomainError::Internal)
    }
}
