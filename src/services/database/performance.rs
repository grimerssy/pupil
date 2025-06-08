use crate::domain::{key::Key, name::Name, performance::KeyLookupError, user_id::DbUserId};

use super::{sql_error, Database};

#[tracing::instrument(skip(db), ret(level = "debug") err(Debug, level = "debug"))]
pub async fn lookup_key(
    db: &Database,
    key: Key,
) -> crate::Result<(DbUserId, Name), KeyLookupError> {
    sqlx::query_as(
        "
        select users.id, users.name
        from users
        join keys on keys.user_id = users.id
        where keys.value = $1
        ",
    )
    .bind(&key)
    .fetch_optional(&db.pool)
    .await
    .map_err(sql_error)?
    .ok_or(crate::Error::expected(KeyLookupError::UnknownKey))
}
