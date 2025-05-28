use crate::app::error::{ContextualError, ErrorContext};

use crate::domain::{
    email::Email,
    name::Name,
    password::{Password, PasswordHash},
};

use super::error::{DomainError, InternalError};

pub trait Signup {
    async fn signup(&self, signup_data: SignupData) -> Result<(), DomainError<SignupError>>;
}

pub trait HashPassword {
    fn hash_password(&self, password: &Password) -> Result<PasswordHash, InternalError>;
}

pub trait SaveNewUser {
    async fn save_new_user(&self, user: NewUser) -> Result<(), DomainError<SaveNewUserError>>;
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
