use crate::{
    context::AppContext,
    domain::{
        auth::{NewUser, SaveNewUser, SaveNewUserError},
        error::{DomainError, DomainResult},
    },
};

use super::{sql_error, Database};

impl SaveNewUser for AppContext {
    #[tracing::instrument(skip(self))]
    async fn save_new_user(&self, new_user: NewUser) -> DomainResult<(), SaveNewUserError> {
        save_new_user_with(&self.database, new_user).await
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
