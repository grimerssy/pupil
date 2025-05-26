use const_fnv1a_hash::fnv1a_hash_str_128;
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use squint::aes::{cipher::KeyInit, Aes128};

use crate::domain::id::Cipher;

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

impl Cipher for IdEncoder {
    fn cipher(&self) -> &Aes128 {
        &self.cipher
    }
}
