use std::time::Duration;

use anyhow::Context;
use jsonwebtoken::{get_current_timestamp, DecodingKey, EncodingKey, Header, Validation};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationSeconds};

use crate::domain::{auth::ParseTokenError, id::UserId, token::AuthToken};

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
struct TokenClaims {
    iat: u64,
    exp: u64,
    user_id: UserId,
}

impl TokenIssuer {
    pub fn new(config: JwtConfig) -> Self {
        Self { config }
    }
}

#[tracing::instrument(skip(issuer), ret(level = "debug") err(Debug, level = "debug"))]
pub fn issue_token(issuer: &TokenIssuer, user_id: UserId) -> crate::Result<AuthToken> {
    let now = get_current_timestamp();
    let claims = TokenClaims {
        iat: now,
        exp: now + issuer.config.ttl.as_secs(),
        user_id,
    };
    let token = jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(issuer.config.secret.expose_secret().as_bytes()),
    )
    .map(AuthToken::new)
    .context("encode jwt")?;
    Ok(token)
}

#[tracing::instrument(skip(issuer), ret(level = "debug") err(Debug, level = "debug"))]
pub fn parse_token(
    issuer: &TokenIssuer,
    token: AuthToken,
) -> crate::Result<UserId, ParseTokenError> {
    let claims = jsonwebtoken::decode::<TokenClaims>(
        &Into::<String>::into(token),
        &DecodingKey::from_secret(issuer.config.secret.expose_secret().as_bytes()),
        &Validation::default(),
    )
    .map(|token| token.claims)
    .map_err(|_| crate::Error::expected(ParseTokenError::Invalid))?;
    Ok(claims.user_id)
}
