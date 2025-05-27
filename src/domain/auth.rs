use crate::{
    app::error::{ContextualError, ErrorContext},
    domain::error::DomainResult,
};

use super::{
    email::{Email, MaybeEmail},
    id::{DbUserId, UserId},
    name::Name,
    password::{MaybePassword, Password, PasswordHash},
    token::AuthToken,
};

#[derive(Debug, Clone)]
pub struct SignupData {
    pub email: Email,
    pub password: Password,
    pub name: Name,
}

pub trait Signup {
    async fn signup(&self, signup_data: SignupData) -> DomainResult<(), SignupError>;
}

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum SignupError {
    HashPasswordError(HashPasswordError),
    SaveNewUserError(SaveNewUserError),
}

impl From<HashPasswordError> for SignupError {
    fn from(value: HashPasswordError) -> Self {
        Self::HashPasswordError(value)
    }
}

impl From<SaveNewUserError> for SignupError {
    fn from(value: SaveNewUserError) -> Self {
        Self::SaveNewUserError(value)
    }
}

impl ContextualError for SignupError {
    fn error_context(self) -> ErrorContext {
        match self {
            Self::HashPasswordError(error) => error.error_context(),
            Self::SaveNewUserError(error) => error.error_context(),
        }
    }
}

pub trait HashPassword {
    fn hash_password(&self, password: Password) -> DomainResult<PasswordHash, HashPasswordError>;
}

#[derive(Debug)]
pub enum HashPasswordError {}
impl ContextualError for HashPasswordError {
    fn error_context(self) -> ErrorContext {
        match self {}
    }
}

#[derive(Debug, Clone)]
pub struct NewUser {
    pub email: Email,
    pub password_hash: PasswordHash,
    pub name: Name,
}

pub trait SaveNewUser {
    async fn save_new_user(&self, user: NewUser) -> DomainResult<(), SaveNewUserError>;
}

#[derive(Debug)]
pub enum SaveNewUserError {
    EmailTaken,
}

impl ContextualError for SaveNewUserError {
    fn error_context(self) -> ErrorContext {
        match self {
            Self::EmailTaken => ErrorContext::new("EMAIL_TAKEN"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoginData {
    pub maybe_email: MaybeEmail,
    pub maybe_password: MaybePassword,
}

pub trait Login {
    async fn login(&self, login_data: LoginData) -> DomainResult<AuthToken, LoginError>;
}

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum LoginError {
    FindUserError(FindUserError),
    VerifyPasswordError(VerifyPasswordError),
    IssueTokenError(IssueTokenError),
}

impl From<FindUserError> for LoginError {
    fn from(value: FindUserError) -> Self {
        Self::FindUserError(value)
    }
}

impl From<VerifyPasswordError> for LoginError {
    fn from(value: VerifyPasswordError) -> Self {
        Self::VerifyPasswordError(value)
    }
}

impl From<IssueTokenError> for LoginError {
    fn from(value: IssueTokenError) -> Self {
        Self::IssueTokenError(value)
    }
}

impl ContextualError for LoginError {
    fn error_context(self) -> ErrorContext {
        match self {
            Self::FindUserError(error) => error.error_context(),
            Self::VerifyPasswordError(error) => error.error_context(),
            Self::IssueTokenError(error) => error.error_context(),
        }
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DatabaseUser {
    pub db_user_id: DbUserId,
    pub password_hash: PasswordHash,
}

pub trait FindUser {
    async fn find_user(&self, email: MaybeEmail) -> DomainResult<DatabaseUser, FindUserError>;
}

#[derive(Debug)]
pub enum FindUserError {
    NotFound,
}
impl ContextualError for FindUserError {
    fn error_context(self) -> ErrorContext {
        match self {
            // TODO
            Self::NotFound => ErrorContext::new("NOT_FOUND"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PasswordClaim {
    pub maybe_password: MaybePassword,
    pub password_hash: PasswordHash,
}

pub trait VerifyPassword {
    fn verify_password(
        &self,
        password_claim: PasswordClaim,
    ) -> DomainResult<(), VerifyPasswordError>;
}

#[derive(Debug)]
pub enum VerifyPasswordError {
    InvalidPassword,
}

impl ContextualError for VerifyPasswordError {
    fn error_context(self) -> ErrorContext {
        match self {
            Self::InvalidPassword => ErrorContext::new("INVALID_PASSWORD"),
        }
    }
}

pub trait IssueToken {
    fn issue_token(&self, user_id: UserId) -> DomainResult<AuthToken, IssueTokenError>;
}

#[derive(Debug)]
pub enum IssueTokenError {}

impl ContextualError for IssueTokenError {
    fn error_context(self) -> ErrorContext {
        match self {}
    }
}
