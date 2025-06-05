use crate::domain::{key::Key, user_id::DbUserId};

use super::{sql_error, Database};

#[tracing::instrument(skip(db), ret(level = "debug") err(Debug, level = "debug"))]
pub async fn get_db_keys(db: &Database, student_id: DbUserId) -> crate::Result<Vec<Key>> {
    sqlx::query_as::<_, (_,)>(
        "
        select value
        from keys
        where user_id = $1
        ",
    )
    .bind(student_id)
    .fetch_all(&db.pool)
    .await
    .map(|keys| keys.into_iter().map(|(key,)| key).collect())
    .map_err(sql_error)
}

#[tracing::instrument(skip(db), ret(level = "debug") err(Debug, level = "debug"))]
pub async fn add_key(db: &Database, student_id: DbUserId, key: Key) -> crate::Result<()> {
    sqlx::query(
        "
        insert into keys
          (value, user_id)
        values
          ($1, $2)
        ",
    )
    .bind(&key)
    .bind(student_id)
    .execute(&db.pool)
    .await
    .map(|_| ())
    .map_err(sql_error)
}

#[tracing::instrument(skip(db), ret(level = "debug") err(Debug, level = "debug"))]
pub async fn remove_db_key(db: &Database, student_id: DbUserId, key: Key) -> crate::Result<()> {
    sqlx::query(
        "
        delete from keys
        where user_id = $1
          and value = $2
        ",
    )
    .bind(student_id)
    .bind(&key)
    .execute(&db.pool)
    .await
    .map(|_| ())
    .map_err(sql_error)
}
