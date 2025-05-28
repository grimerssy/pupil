use crate::{
    app::error::log_result,
    domain::{
        error::DomainError,
        id::{Cipher, UserId},
        login::{FindUser, IssueToken, Login, LoginData, LoginError, VerifyPassword},
        signup::{HashPassword, NewUser, SaveNewUser, Signup, SignupData, SignupError},
        token::AuthToken,
    },
};

use super::{
    error::{AppError, AppErrorKind},
    validation::ConversionFailure,
    AppContext,
};

#[tracing::instrument(skip(ctx))]
pub async fn signup<T>(ctx: &AppContext, form: T) -> Result<(), AppError<T, SignupError>>
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

#[tracing::instrument(skip(ctx))]
pub async fn login<T>(ctx: &AppContext, form: T) -> Result<AuthToken, AppError<T, LoginError>>
where
    T: Clone + core::fmt::Debug,
    LoginData: TryFrom<T, Error = ConversionFailure<T>>,
{
    log_result!(async {
        let login_data = LoginData::try_from(form.clone()).map_err(AppError::from)?;
        ctx.login(login_data)
            .await
            .map_err(AppErrorKind::Logical)
            .map_err(|error| error.with_input(form))
    })
}

impl Signup for AppContext {
    async fn signup(&self, signup_data: SignupData) -> Result<(), DomainError<SignupError>> {
        signup_with(&self.hasher, &self.database, signup_data).await
    }
}

impl Login for AppContext {
    async fn login(&self, login_data: LoginData) -> Result<AuthToken, DomainError<LoginError>> {
        login_with(
            &self.database,
            &self.hasher,
            self.id_encoder.as_ref(),
            &self.token_issuer,
            login_data,
        )
        .await
    }
}

async fn signup_with(
    hasher: &impl HashPassword,
    storage: &impl SaveNewUser,
    signup_data: SignupData,
) -> Result<(), DomainError<SignupError>> {
    let SignupData {
        email,
        name,
        password,
    } = signup_data;
    let password_hash = hasher
        .hash_password(&password)
        .map_err(DomainError::from_internal)?;
    let new_user = NewUser {
        email,
        name,
        password_hash,
    };
    storage
        .save_new_user(new_user)
        .await
        .map_err(DomainError::cast)
}

async fn login_with(
    storage: &impl FindUser,
    verifier: &impl VerifyPassword,
    cipher: &impl Cipher,
    issuer: &impl IssueToken,
    login_data: LoginData,
) -> Result<AuthToken, DomainError<LoginError>> {
    let user = storage
        .find_user(&login_data.email)
        .await
        .map_err(DomainError::cast)?;
    verifier
        .verify_password(login_data.password, user.password_hash)
        .map_err(DomainError::cast)?;
    let user_id = UserId::new(user.db_user_id, cipher.cipher());
    issuer
        .issue_token(user_id)
        .map_err(DomainError::from_internal)
}
