use crate::{
    domain::{
        auth::DecodeUserId,
        key::Key,
        keys::*,
        user_id::{DbUserId, UserId},
    },
    error::ErrorKind,
    services::database::keys::{add_key, get_db_keys, remove_db_key},
};

use super::AppContext;

impl GetKeys for AppContext {
    #[tracing::instrument(skip(self), ret(level = "debug") err(Debug, level = "debug"))]
    async fn get_keys(&self, student_id: UserId) -> crate::Result<Vec<Key>> {
        get_keys_with(self, self, student_id).await
    }
}

impl GenerateKey for AppContext {
    #[tracing::instrument(skip(self), ret(level = "debug") err(Debug, level = "debug"))]
    async fn generate_key(&self, student_id: UserId) -> crate::Result<Vec<Key>, GenerateKeyError> {
        generate_key_with(self, self, self, student_id).await
    }
}

#[tracing::instrument(skip(ctx), ret(level = "debug") err(Debug, level = "debug"))]
pub async fn remove_key(
    ctx: &AppContext,
    student_id: UserId,
    key: String,
) -> crate::Result<Vec<Key>, RemoveKeyError> {
    let key = Key::try_from(key).map_err(|_| crate::Error::expected(RemoveKeyError::UnknownKey))?;
    ctx.remove_key(student_id, key).await
}

impl RemoveKey for AppContext {
    #[tracing::instrument(skip(self), ret(level = "debug") err(Debug, level = "debug"))]
    async fn remove_key(
        &self,
        student_id: UserId,
        key: Key,
    ) -> crate::Result<Vec<Key>, RemoveKeyError> {
        remove_key_with(self, self, self, student_id, key).await
    }
}

impl GetDbKeys for AppContext {
    async fn get_db_keys(&self, student_id: DbUserId) -> crate::Result<Vec<Key>> {
        get_db_keys(&self.database, student_id).await
    }
}

impl AddKey for AppContext {
    async fn add_key(&self, student_id: DbUserId, key: Key) -> crate::Result<()> {
        add_key(&self.database, student_id, key).await
    }
}

impl RemoveDbKey for AppContext {
    async fn remove_db_key(&self, student_id: DbUserId, key: Key) -> crate::Result<()> {
        remove_db_key(&self.database, student_id, key).await
    }
}

async fn get_keys_with(
    decoder: &impl DecodeUserId,
    storage: &impl GetDbKeys,
    student_id: UserId,
) -> crate::Result<Vec<Key>> {
    let student_id = match decoder.decode_user_id(student_id) {
        Ok(id) => id,
        Err(error) => match error.kind {
            ErrorKind::Expected(_) => return Ok(Vec::new()),
            ErrorKind::Internal(error) => return Err(crate::Error::internal(error)),
        },
    };
    storage.get_db_keys(student_id).await
}

async fn generate_key_with(
    decoder: &impl DecodeUserId,
    adder: &impl AddKey,
    storage: &impl GetDbKeys,
    student_id: UserId,
) -> crate::Result<Vec<Key>, GenerateKeyError> {
    let student_id = decoder
        .decode_user_id(student_id)
        .map_err(|_| crate::Error::expected(GenerateKeyError::UnknownUser))?;
    let key = Key::new();
    adder
        .add_key(student_id, key)
        .await
        .map_err(crate::Error::from_internal)?;
    storage
        .get_db_keys(student_id)
        .await
        .map_err(crate::Error::from_internal)
}

async fn remove_key_with(
    decoder: &impl DecodeUserId,
    remover: &impl RemoveDbKey,
    storage: &impl GetDbKeys,
    student_id: UserId,
    key: Key,
) -> crate::Result<Vec<Key>, RemoveKeyError> {
    let student_id = decoder
        .decode_user_id(student_id)
        .map_err(|_| crate::Error::expected(RemoveKeyError::UnknownKey))?;
    remover
        .remove_db_key(student_id, key)
        .await
        .map_err(crate::Error::from_internal)?;
    storage
        .get_db_keys(student_id)
        .await
        .map_err(crate::Error::from_internal)
}
