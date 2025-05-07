use crate::{
    domain::{
        auth::{NewUser, SaveNewUser, SaveNewUserError},
        error::{DomainError, DomainResult},
    },
    AppContext,
};

use super::{sql_error, Database};

pub fn save_new_user_with(ctx: &AppContext) -> impl SaveNewUser {
    async |new_user| save_new_user(&ctx.database, new_user).await
}

#[tracing::instrument(skip(db))]
async fn save_new_user(db: &Database, new_user: NewUser) -> DomainResult<(), SaveNewUserError> {
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
