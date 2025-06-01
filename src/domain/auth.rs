use serde::Serialize;

use crate::app::localization::LocalizedError;

use crate::domain::{
    email::{Email, MaybeEmail},
    user_id::{DbUserId, UserId},
    name::Name,
    password::{MaybePassword, Password, PasswordHash},
    token::AuthToken,
};

use super::role::Role;

pub trait Signup {
    async fn signup(&self, signup_data: SignupData) -> crate::Result<(), SignupError>;
}

pub trait Login {
    async fn login(&self, login_data: LoginData) -> crate::Result<AuthToken, LoginError>;
}

pub trait Authenticate {
    async fn authenticate(&self, token: AuthToken) -> crate::Result<User, AuthError>;
}

pub trait SaveNewUser {
    async fn save_new_user(&self, user: NewUser) -> crate::Result<(), SaveNewUserError>;
}

pub trait HashPassword {
    fn hash_password(&self, password: &Password) -> crate::Result<PasswordHash>;
}

pub trait FindUser {
    async fn find_user(&self, email: &MaybeEmail) -> crate::Result<DbUser, FindUserError>;
}

pub trait GetUser {
    async fn get_user(&self, db_id: &DbUserId) -> crate::Result<DbUser, GetUserError>;
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

pub trait ParseToken {
    fn parse_token(&self, token: AuthToken) -> crate::Result<UserId, ParseTokenError>;
}

pub trait EncodeUserId {
    fn encode_user_id(&self, raw_id: DbUserId) -> crate::Result<UserId>;
}

pub trait DecodeUserId {
    fn decode_user_id(&self, id: UserId) -> crate::Result<DbUserId, DecodeIdError>;
}

#[derive(Debug, Clone)]
pub struct SignupData {
    pub email: Email,
    pub password: Password,
    pub name: Name,
}

#[derive(Debug, Clone)]
pub struct LoginData {
    pub email: MaybeEmail,
    pub password: MaybePassword,
}

#[derive(Debug, Clone)]
pub struct NewUser {
    pub email: Email,
    pub password_hash: PasswordHash,
    pub name: Name,
    pub role: Role,
}

#[derive(Debug, Clone, Serialize)]
pub struct User {
    pub id: UserId,
    pub email: Email,
    pub name: Name,
    pub role: Role,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DbUser {
    pub id: DbUserId,
    pub email: Email,
    pub name: Name,
    pub password_hash: PasswordHash,
    pub role: Role,
}

#[derive(Debug)]
pub enum SignupError {
    EmailTaken,
}

#[derive(Debug)]
pub enum LoginError {
    InvalidCredentials,
}

#[derive(Debug)]
pub enum AuthError {
    Unauthenticated,
}

#[derive(Debug)]
pub enum SaveNewUserError {
    EmailConflict,
}

#[derive(Debug)]
pub enum FindUserError {
    NotFound,
}

#[derive(Debug)]
pub enum VerifyPasswordError {
    InvalidPassword,
}

#[derive(Debug)]
pub enum GetUserError {
    NotFound,
}

#[derive(Debug)]
pub enum ParseTokenError {
    Invalid,
}

#[derive(Debug)]
pub enum DecodeIdError {
    InvalidFormat,
}

impl From<SignupError> for LocalizedError {
    fn from(value: SignupError) -> Self {
        match value {
            SignupError::EmailTaken => Self::new("EMAIL_TAKEN"),
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

impl From<SaveNewUserError> for SignupError {
    fn from(value: SaveNewUserError) -> Self {
        match value {
            SaveNewUserError::EmailConflict => Self::EmailTaken,
        }
    }
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

impl From<GetUserError> for AuthError {
    fn from(value: GetUserError) -> Self {
        match value {
            GetUserError::NotFound => Self::Unauthenticated,
        }
    }
}

impl From<ParseTokenError> for AuthError {
    fn from(value: ParseTokenError) -> Self {
        match value {
            ParseTokenError::Invalid => Self::Unauthenticated,
        }
    }
}

impl From<DecodeIdError> for AuthError {
    fn from(value: DecodeIdError) -> Self {
        match value {
            DecodeIdError::InvalidFormat => Self::Unauthenticated,
        }
    }
}
