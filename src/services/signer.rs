use std::sync::{Arc, Mutex};

use anyhow::Context;
use ed25519_dalek::{ed25519::signature::SignerMut, SecretKey, SigningKey};
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use serde_json_canonicalizer as jcs;

use crate::domain::{
    performance::PerformanceEvaluation, signature::Signature, verifying_key::VerifyingKey,
};

#[derive(Clone, Debug, Deserialize)]
pub struct SignatureConfig {
    secret: SecretString,
}

#[derive(Clone)]
pub struct Signer {
    key: Arc<Mutex<SigningKey>>,
}

impl Signer {
    pub fn new(config: SignatureConfig) -> anyhow::Result<Self> {
        let key_bytes = SecretKey::try_from(config.secret.expose_secret().as_bytes())
            .context("get key bytes from secret")?;
        let key = SigningKey::from_bytes(&key_bytes);
        Ok(Self {
            key: Arc::new(Mutex::new(key)),
        })
    }
}

#[tracing::instrument(skip(signer), ret(level = "debug") err(Debug, level = "debug"))]
pub fn sign_evaluation(signer: &Signer, claim: &PerformanceEvaluation) -> crate::Result<Signature> {
    let bytes = jcs::to_vec(claim).context("serialize performance claim")?;
    let mut key = signer.key.lock().unwrap();
    let signature = key.try_sign(&bytes).context("sign the claim")?;
    Ok(Signature::new(signature))
}

#[tracing::instrument(skip(signer), ret(level = "debug") err(Debug, level = "debug"))]
pub fn get_verifying_key(signer: &Signer) -> crate::Result<VerifyingKey> {
    let key = signer.key.lock().unwrap().verifying_key();
    Ok(VerifyingKey::new(key))
}
