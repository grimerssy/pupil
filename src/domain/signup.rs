use crate::app::localization::LocalizedError;

use crate::domain::{
    email::Email,
    name::Name,
    password::{Password, PasswordHash},
};

pub trait Signup {
    async fn signup(&self, signup_data: SignupData) -> crate::Result<(), SignupError>;
}

pub trait HashPassword {
    fn hash_password(&self, password: &Password) -> crate::Result<PasswordHash>;
}

pub trait SaveNewUser {
    async fn save_new_user(&self, user: NewUser) -> crate::Result<(), SaveNewUserError>;
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

impl From<SignupError> for LocalizedError {
    fn from(value: SignupError) -> Self {
        match value {
            SignupError::EmailTaken => Self::new("EMAIL_TAKEN"),
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
