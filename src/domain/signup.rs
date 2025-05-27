use std::convert::Infallible;

use crate::{
    app::error::{ContextualError, ErrorContext},
    domain::error::DomainResult,
};

use crate::domain::{
    email::Email,
    name::Name,
    password::{Password, PasswordHash},
};

pub trait Signup {
    async fn signup(&self, signup_data: SignupData) -> DomainResult<(), SignupError>;
}

pub trait HashPassword {
    fn hash_password(&self, password: &Password) -> DomainResult<PasswordHash, Infallible>;
}

pub trait SaveNewUser {
    async fn save_new_user(&self, user: NewUser) -> DomainResult<(), SaveNewUserError>;
}

#[derive(Debug, Clone)]
pub struct SignupData {
    pub email: Email,
    pub password: Password,
    pub name: Name,
}

#[derive(Debug, Clone)]
pub struct NewUser {
    pub email: Email,
    pub password_hash: PasswordHash,
    pub name: Name,
}

#[derive(Debug)]
pub enum SignupError {
    EmailTaken,
}

#[derive(Debug)]
pub enum SaveNewUserError {
    EmailConflict,
}

impl ContextualError for SignupError {
    fn error_context(self) -> ErrorContext {
        match self {
            Self::EmailTaken => ErrorContext::new("EMAIL_TAKEN"),
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
