use crate::{
    app::error::log_result,
    context::AppContext,
    domain::{
        auth::{HashPassword, NewUser, SaveNewUser, Signup, SignupData, SignupError},
        error::{DomainError, DomainResult},
    },
};

use super::{
    error::{AppError, AppErrorKind, AppResult},
    validation::ConversionFailure,
};

#[tracing::instrument(skip(ctx))]
pub async fn signup<T>(ctx: &AppContext, form: T) -> AppResult<T, (), SignupError>
where
    T: Clone + core::fmt::Debug,
    SignupData: TryFrom<T, Error = ConversionFailure<T>>,
{
    log_result!(async {
        let signup_data = SignupData::try_from(form.clone()).map_err(AppError::from)?;
        ctx.signup(signup_data)
            .await
            .map_err(AppErrorKind::Logical)
            .map_err(|error| error.with_input(form))
    })
}

impl Signup for &AppContext {
    async fn signup(self, signup_data: SignupData) -> DomainResult<(), SignupError> {
        signup_with(self, self, signup_data).await
    }
}

async fn signup_with(
    hasher: impl HashPassword,
    db: impl SaveNewUser,
    signup_data: SignupData,
) -> DomainResult<(), SignupError> {
    let SignupData {
        email,
        name,
        password,
    } = signup_data;
    let password_hash = hasher.hash_password(password).map_err(DomainError::cast)?;
    let new_user = NewUser {
        email,
        name,
        password_hash,
    };
    db.save_new_user(new_user).await.map_err(DomainError::cast)
}
