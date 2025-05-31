use const_fnv1a_hash::fnv1a_hash_str_128;
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use squint::{
    aes::{cipher::KeyInit, Aes128},
    Id,
};

use crate::domain::{
    auth::DecodeIdError,
    id::{DbUserId, UserId},
};

#[derive(Clone, Debug, Deserialize)]
pub struct IdConfig {
    secret: SecretString,
}

#[derive(Clone)]
pub struct IdEncoder {
    cipher: Aes128,
}

impl IdEncoder {
    pub fn new(config: IdConfig) -> Self {
        let key = fnv1a_hash_str_128(config.secret.expose_secret()).to_le_bytes();
        let cipher = Aes128::new(&key.into());
        Self { cipher }
    }
}

pub fn encode_user_id(encoder: &IdEncoder, raw_id: DbUserId) -> crate::Result<UserId> {
    let id = Id::new(raw_id.into(), &encoder.cipher);
    Ok(UserId::new(id))
}

pub fn decode_user_id(encoder: &IdEncoder, id: UserId) -> crate::Result<DbUserId, DecodeIdError> {
    let id: Id<{ squint::tag("user") }> = id.into();
    id.to_raw(&encoder.cipher)
        .map(DbUserId::new)
        .map_err(|_| crate::Error::expected(DecodeIdError::InvalidFormat))
}
