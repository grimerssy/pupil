use crate::app::localization::LocalizedError;

use super::{
    key::Key,
    user_id::{DbUserId, UserId},
};

pub trait GetKeys {
    async fn get_keys(&self, student_id: UserId) -> crate::Result<Vec<Key>>;
}

pub trait GenerateKey {
    async fn generate_key(&self, student_id: UserId) -> crate::Result<Vec<Key>, GenerateKeyError>;
}

pub trait RemoveKey {
    async fn remove_key(
        &self,
        student_id: UserId,
        key: Key,
    ) -> crate::Result<Vec<Key>, RemoveKeyError>;
}

pub trait GetDbKeys {
    async fn get_db_keys(&self, student_id: DbUserId) -> crate::Result<Vec<Key>>;
}

pub trait AddKey {
    async fn add_key(&self, student_id: DbUserId, key: Key) -> crate::Result<()>;
}

pub trait RemoveDbKey {
    async fn remove_db_key(&self, student_id: DbUserId, key: Key) -> crate::Result<()>;
}

#[derive(Debug)]
pub enum GenerateKeyError {
    UnknownUser,
}

#[derive(Debug)]
pub enum RemoveKeyError {
    UnknownKey,
}

impl From<GenerateKeyError> for LocalizedError {
    fn from(value: GenerateKeyError) -> Self {
        match value {
            GenerateKeyError::UnknownUser => Self::new("UNAUTHORIZED"),
        }
    }
}

impl From<RemoveKeyError> for LocalizedError {
    fn from(value: RemoveKeyError) -> Self {
        match value {
            RemoveKeyError::UnknownKey => Self::new("NOT_FOUND"),
        }
    }
}
