use thiserror::Error;

use super::{
    action::action,
    email::Email,
    error::DomainResult,
    name::Name,
    password::{Password, PasswordHash},
};

action! {
    pub Signup = async (SignupData) -> DomainResult<(), SignupError>;

    pub HashPassword = (Password) -> DomainResult<PasswordHash, HashPasswordError>;
    pub SaveNewUser = async (NewUser) -> DomainResult<(), SaveNewUserError>;
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
