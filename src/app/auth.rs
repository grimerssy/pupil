use crate::{
    app::error::log_result,
    domain::{
        auth::{
            FindUser, HashPassword, IssueToken, Login, LoginData, LoginError, NewUser,
            PasswordClaim, SaveNewUser, Signup, SignupData, SignupError, VerifyPassword,
        },
        error::{DomainError, DomainResult},
        id::{Cipher, UserId},
        token::AuthToken,
    },
};

use super::{
    error::{AppError, AppErrorKind, AppResult},
    validation::ConversionFailure,
    AppContext,
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

#[tracing::instrument(skip(ctx))]
pub async fn login<T>(ctx: &AppContext, form: T) -> AppResult<T, AuthToken, LoginError>
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
    async fn signup(&self, signup_data: SignupData) -> DomainResult<(), SignupError> {
        signup_with(&self.hasher, &self.database, signup_data).await
    }
}

impl Login for AppContext {
    async fn login(&self, login_data: LoginData) -> DomainResult<AuthToken, LoginError> {
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
) -> DomainResult<AuthToken, LoginError> {
    let user = storage
        .find_user(login_data.maybe_email)
        .await
        .map_err(DomainError::cast)?;
    let password_claim = PasswordClaim {
        maybe_password: login_data.maybe_password,
        password_hash: user.password_hash,
    };
    verifier
        .verify_password(password_claim)
        .map_err(DomainError::cast)?;
    let user_id = UserId::new(user.db_user_id, cipher.cipher());
    issuer.issue_token(user_id).map_err(DomainError::cast)
}
