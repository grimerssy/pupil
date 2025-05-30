use crate::{
    app::AppError,
    domain::{
        auth::{
            DatabaseUser, EncodeUserId, FindUser, FindUserError, HashPassword, IssueToken, Login,
            LoginData, LoginError, NewUser, SaveNewUser, SaveNewUserError, Signup, SignupData,
            SignupError, VerifyPassword, VerifyPasswordError,
        },
        email::MaybeEmail,
        id::UserId,
        password::{MaybePassword, Password, PasswordHash},
        role::Role,
        token::AuthToken,
    },
    services::{
        database::auth::{find_user, save_new_user},
        hasher::{hash_password, verify_password},
        id_encoder::encode_user_id,
        token_issuer::issue_token,
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
    ctx.signup(signup_data).await.map_err(crate::Error::cast)
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
    let role = Role::Student;
    let new_user = NewUser {
        email,
        name,
        password_hash,
        role,
    };
    storage
        .save_new_user(new_user)
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

async fn login_with(
    storage: &impl FindUser,
    verifier: &impl VerifyPassword,
    encoder: &impl EncodeUserId,
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
    let user_id = encoder
        .encode_user_id(user.id)
        .map_err(crate::Error::from_internal)?;
    issuer
        .issue_token(user_id)
        .map_err(crate::Error::from_internal)
}

impl Signup for AppContext {
    async fn signup(&self, signup_data: SignupData) -> crate::Result<(), SignupError> {
        signup_with(self, self, signup_data).await
    }
}

impl Login for AppContext {
    async fn login(&self, login_data: LoginData) -> crate::Result<AuthToken, LoginError> {
        login_with(self, self, self, self, login_data).await
    }
}

impl FindUser for AppContext {
    async fn find_user(&self, email: &MaybeEmail) -> crate::Result<DatabaseUser, FindUserError> {
        find_user(&self.database, email).await
    }
}

impl SaveNewUser for AppContext {
    async fn save_new_user(&self, new_user: NewUser) -> crate::Result<(), SaveNewUserError> {
        save_new_user(&self.database, new_user).await
    }
}

impl HashPassword for AppContext {
    fn hash_password(&self, password: &Password) -> crate::Result<PasswordHash> {
        hash_password(&self.hasher, password)
    }
}

impl VerifyPassword for AppContext {
    fn verify_password(
        &self,
        password: MaybePassword,
        password_hash: PasswordHash,
    ) -> crate::Result<(), VerifyPasswordError> {
        verify_password(&self.hasher, password, password_hash)
    }
}

impl IssueToken for AppContext {
    fn issue_token(&self, user_id: UserId) -> crate::Result<AuthToken> {
        issue_token(&self.token_issuer, user_id)
    }
}

impl EncodeUserId for AppContext {
    fn encode_user_id(&self, raw_id: crate::domain::id::DbUserId) -> crate::Result<UserId> {
        Ok(encode_user_id(&self.id_encoder, raw_id))
    }
}
