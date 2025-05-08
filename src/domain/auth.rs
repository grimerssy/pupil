use thiserror::Error;

use super::{
    email::Email,
    error::DomainResult,
    name::Name,
    password::{Password, PasswordHash},
};

pub trait Signup {
    async fn signup(self, input: SignupData) -> DomainResult<(), SignupError>;
}

pub trait HashPassword {
    fn hash_password(self, input: Password) -> DomainResult<PasswordHash, HashPasswordError>;
}

pub trait SaveNewUser {
    async fn save_new_user(self, input: NewUser) -> DomainResult<(), SaveNewUserError>;
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

#[derive(Debug, Error)]
pub enum SignupError {
    #[error(transparent)]
    SaveUser(#[from] SaveNewUserError),
    #[error(transparent)]
    HashPassword(#[from] HashPasswordError),
}

#[derive(Debug, Error)]
pub enum HashPasswordError {}

#[derive(Debug, Error)]
pub enum SaveNewUserError {
    #[error("Email address is already in use")]
    EmailTaken,
}
