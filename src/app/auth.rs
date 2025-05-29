use crate::{
    app::AppError,
    domain::{
        id::{Cipher, UserId},
        login::{FindUser, IssueToken, Login, LoginData, LoginError, VerifyPassword},
        signup::{HashPassword, NewUser, SaveNewUser, Signup, SignupData, SignupError},
        token::AuthToken,
    },
};

use super::{validation::ValidationErrors, AppContext};

#[tracing::instrument(skip(ctx), ret(level = "debug") err(Debug, level = "debug"))]
pub async fn signup<T>(ctx: &AppContext, form: T) -> crate::Result<(), AppError<SignupError>>
where
    T: core::fmt::Debug + TryInto<SignupData, Error = ValidationErrors>,
{
    let signup_data = form
        .try_into()
        .map_err(AppError::Validation)
        .map_err(crate::Error::expected)?;
    ctx.signup(signup_data)
        .await
        .map_err(crate::Error::cast)
}

#[tracing::instrument(skip(ctx), ret(level = "debug") err(Debug, level = "debug"))]
pub async fn login<T>(ctx: &AppContext, form: T) -> crate::Result<AuthToken, AppError<LoginError>>
where
    T: core::fmt::Debug + TryInto<LoginData, Error = ValidationErrors>,
{
    let data = form
        .try_into()
        .map_err(AppError::Validation)
        .map_err(crate::Error::expected)?;
    ctx.login(data).await.map_err(crate::Error::cast)
}

impl Signup for AppContext {
    async fn signup(&self, signup_data: SignupData) -> crate::Result<(), SignupError> {
        signup_with(&self.hasher, &self.database, signup_data).await
    }
}

impl Login for AppContext {
    async fn login(&self, login_data: LoginData) -> crate::Result<AuthToken, LoginError> {
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
) -> crate::Result<(), SignupError> {
    let SignupData {
        email,
        name,
        password,
    } = signup_data;
    let password_hash = hasher
        .hash_password(&password)
        .map_err(crate::Error::from_internal)?;
    let new_user = NewUser {
        email,
        name,
        password_hash,
    };
    storage
        .save_new_user(new_user)
        .await
        .map_err(crate::Error::cast)
}

async fn login_with(
    storage: &impl FindUser,
    verifier: &impl VerifyPassword,
    cipher: &impl Cipher,
    issuer: &impl IssueToken,
    login_data: LoginData,
) -> crate::Result<AuthToken, LoginError> {
    let user = storage
        .find_user(&login_data.email)
        .await
        .map_err(crate::Error::cast)?;
    verifier
        .verify_password(login_data.password, user.password_hash)
        .map_err(crate::Error::cast)?;
    let user_id = UserId::new(user.db_user_id, cipher.cipher());
    issuer
        .issue_token(user_id)
        .map_err(crate::Error::from_internal)
}
