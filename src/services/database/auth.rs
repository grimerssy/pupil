use crate::domain::{
    auth::{FindUserError, GetUserError, NewUser, SaveNewUserError, DbUser},
    email::MaybeEmail,
    id::DbUserId,
};

use super::{sql_error, Database};

#[tracing::instrument(skip(db), ret(level = "debug") err(Debug, level = "debug"))]
pub async fn save_new_user(
    db: &Database,
    new_user: NewUser,
) -> crate::Result<(), SaveNewUserError> {
    match sqlx::query(
        "
        insert into users
          (email, name, password_hash, role)
        values
          ($1, $2, $3, $4)
        ",
    )
    .bind(new_user.email)
    .bind(new_user.name)
    .bind(new_user.password_hash)
    .bind(new_user.role)
    .execute(&db.pool)
    .await
    {
        Err(sqlx::Error::Database(error)) if error.is_unique_violation() => {
            Err(crate::Error::expected(SaveNewUserError::EmailConflict))
        }
        result => result.map(|_| ()).map_err(sql_error),
    }
}

#[tracing::instrument(skip(db), ret(level = "debug") err(Debug, level = "debug"))]
pub async fn get_user(db: &Database, db_id: &DbUserId) -> crate::Result<DbUser, GetUserError> {
    sqlx::query_as(
        "
        select id, email, name, password_hash, role
        from users
        where id = $1
        ",
    )
    .bind(db_id)
    .fetch_optional(&db.pool)
    .await
    .map_err(sql_error)?
    .ok_or(crate::Error::expected(GetUserError::NotFound))
}

#[tracing::instrument(skip(db), ret(level = "debug") err(Debug, level = "debug"))]
pub async fn find_user(db: &Database, email: &MaybeEmail) -> crate::Result<DbUser, FindUserError> {
    sqlx::query_as(
        "
        select id, email, name, password_hash, role
        from users
        where email = $1
        ",
    )
    .bind(email)
    .fetch_optional(&db.pool)
    .await
    .map_err(sql_error)?
    .ok_or(crate::Error::expected(FindUserError::NotFound))
}
