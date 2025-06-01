use crate::{
    app::AppError,
    domain::{
        auth::*,
        email::MaybeEmail,
        user_id::{DbUserId, UserId},
        password::{MaybePassword, Password, PasswordHash},
        role::Role,
        token::AuthToken,
    },
    services::{
        database::auth::{find_user, get_user, save_new_user},
        hasher::{hash_password, verify_password},
        id_encoder::{decode_user_id, encode_user_id},
        token_issuer::{issue_token, parse_token},
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

#[tracing::instrument(skip(ctx), ret(level = "debug") err(Debug, level = "debug"))]
pub async fn authenticate(ctx: &AppContext, token: AuthToken) -> crate::Result<User, AuthError> {
    ctx.authenticate(token).await
}

async fn authenticate_with(
    parser: &impl ParseToken,
    encoder: &impl EncodeUserId,
    decoder: &impl DecodeUserId,
    storage: &impl GetUser,
    token: AuthToken,
) -> crate::Result<User, AuthError> {
    let user_id = parser.parse_token(token).map_err(crate::Error::cast)?;
    let db_id = decoder
        .decode_user_id(user_id)
        .map_err(crate::Error::cast)?;
    let DbUser {
        id,
        email,
        name,
        password_hash: _,
        role,
    } = storage.get_user(&db_id).await.map_err(crate::Error::cast)?;
    let id = encoder
        .encode_user_id(id)
        .map_err(crate::Error::from_internal)?;
    Ok(User {
        id,
        email,
        name,
        role,
    })
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

impl Authenticate for AppContext {
    async fn authenticate(&self, token: AuthToken) -> crate::Result<User, AuthError> {
        authenticate_with(self, self, self, self, token).await
    }
}

impl GetUser for AppContext {
    async fn get_user(&self, db_id: &DbUserId) -> crate::Result<DbUser, GetUserError> {
        get_user(&self.database, db_id).await
    }
}

impl FindUser for AppContext {
    async fn find_user(&self, email: &MaybeEmail) -> crate::Result<DbUser, FindUserError> {
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

impl ParseToken for AppContext {
    fn parse_token(&self, token: AuthToken) -> crate::Result<UserId, ParseTokenError> {
        parse_token(&self.token_issuer, token)
    }
}

impl EncodeUserId for AppContext {
    fn encode_user_id(&self, raw_id: DbUserId) -> crate::Result<UserId> {
        encode_user_id(&self.id_encoder, raw_id)
    }
}

impl DecodeUserId for AppContext {
    fn decode_user_id(&self, id: UserId) -> crate::Result<DbUserId, DecodeIdError> {
        decode_user_id(&self.id_encoder, id)
    }
}
