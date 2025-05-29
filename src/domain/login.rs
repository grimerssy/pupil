use crate::app::localization::LocalizedError;

use crate::domain::{
    email::MaybeEmail,
    id::{DbUserId, UserId},
    password::{MaybePassword, PasswordHash},
    token::AuthToken,
};

pub trait Login {
    async fn login(&self, login_data: LoginData) -> crate::Result<AuthToken, LoginError>;
}

pub trait FindUser {
    async fn find_user(&self, email: &MaybeEmail) -> crate::Result<DatabaseUser, FindUserError>;
}

pub trait VerifyPassword {
    fn verify_password(
        &self,
        password: MaybePassword,
        password_hash: PasswordHash,
    ) -> crate::Result<(), VerifyPasswordError>;
}

pub trait IssueToken {
    fn issue_token(&self, user_id: UserId) -> crate::Result<AuthToken>;
}

#[derive(Debug, Clone)]
pub struct LoginData {
    pub email: MaybeEmail,
    pub password: MaybePassword,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DatabaseUser {
    pub db_user_id: DbUserId,
    pub password_hash: PasswordHash,
}

#[derive(Debug)]
pub enum LoginError {
    InvalidCredentials,
}

#[derive(Debug)]
pub enum FindUserError {
    NotFound,
}

#[derive(Debug)]
pub enum VerifyPasswordError {
    InvalidPassword,
}

impl From<FindUserError> for LoginError {
    fn from(value: FindUserError) -> Self {
        match value {
            FindUserError::NotFound => Self::InvalidCredentials,
        }
    }
}

impl From<VerifyPasswordError> for LoginError {
    fn from(value: VerifyPasswordError) -> Self {
        match value {
            VerifyPasswordError::InvalidPassword => Self::InvalidCredentials,
        }
    }
}

impl From<LoginError> for LocalizedError {
    fn from(value: LoginError) -> Self {
        match value {
            LoginError::InvalidCredentials => Self::new("INVALID_CREDENTIALS"),
        }
    }
}
