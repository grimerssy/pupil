use crate::domain::{
    auth::{DatabaseUser, FindUser, FindUserError, NewUser, SaveNewUser, SaveNewUserError},
    email::MaybeEmail,
    error::{DomainError, DomainResult},
};

use super::{sql_error, Database};

impl SaveNewUser for Database {
    #[tracing::instrument(skip(self))]
    async fn save_new_user(&self, new_user: NewUser) -> DomainResult<(), SaveNewUserError> {
        save_new_user_with(self, new_user).await
    }
}

impl FindUser for Database {
    #[tracing::instrument(skip(self))]
    async fn find_user(&self, email: MaybeEmail) -> DomainResult<DatabaseUser, FindUserError> {
        find_user_with(self, &email).await
    }
}

async fn save_new_user_with(
    db: &Database,
    new_user: NewUser,
) -> DomainResult<(), SaveNewUserError> {
    match sqlx::query(
        "
        insert into users
          (email, name, password_hash)
        values
          ($1, $2, $3)
        ",
    )
    .bind(new_user.email)
    .bind(new_user.name)
    .bind(new_user.password_hash)
    .execute(&db.pool)
    .await
    {
        Err(sqlx::Error::Database(error)) if error.is_unique_violation() => {
            Err(DomainError::Expected(SaveNewUserError::EmailTaken))
        }
        result => result.map(|_| ()).map_err(sql_error),
    }
}

async fn find_user_with(
    db: &Database,
    email: &MaybeEmail,
) -> DomainResult<DatabaseUser, FindUserError> {
    sqlx::query_as(
        "
        select id as db_user_id, password_hash
        from users
        where email = $1
        ",
    )
    .bind(email)
    .fetch_optional(&db.pool)
    .await
    .map_err(sql_error)?
    .ok_or(DomainError::Expected(FindUserError::NotFound))
}
