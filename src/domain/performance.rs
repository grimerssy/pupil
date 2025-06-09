use serde::Serialize;

use crate::app::localization::LocalizedError;

use super::{
    key::Key, name::Name, percentile::Percentile, signature::Signature, user_id::DbUserId,
    verifying_key::VerifyingKey,
};

pub trait GetSignature {
    async fn get_signature(&self, key: Key) -> crate::Result<SignedEvaluation, KeyLookupError>;
}

pub trait GetVerifyingKey {
    fn get_verifying_key(&self) -> crate::Result<VerifyingKey>;
}

pub trait GetPerformanceEvaluation {
    async fn get_performance_evaluation(
        &self,
        key: Key,
    ) -> crate::Result<PerformanceEvaluation, KeyLookupError>;
}

pub trait SignEvaluation {
    fn sign_evaluation(&self, claim: &PerformanceEvaluation) -> crate::Result<Signature>;
}

pub trait LookupKey {
    async fn lookup_key(&self, key: Key) -> crate::Result<(DbUserId, Name), KeyLookupError>;
}

#[derive(Debug, Clone, Serialize)]
pub struct SignedEvaluation {
    pub claim: PerformanceEvaluation,
    pub signature: Signature,
}

#[derive(Debug, Clone, Serialize)]
pub struct PerformanceEvaluation {
    pub student: Name,
    pub percentile: Percentile,
}

#[derive(Debug)]
pub enum KeyLookupError {
    UnknownKey,
}

impl From<KeyLookupError> for LocalizedError {
    fn from(value: KeyLookupError) -> Self {
        match value {
            KeyLookupError::UnknownKey => Self::new("UNKNOWN_KEY"),
        }
    }
}
