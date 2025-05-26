#![allow(unused)]

use super::prelude::*;

define! {
    record SignupData = (email, password, name);

    operation Signup = async (SignupData) -> { (), SignupError };

    error SignupError = {
        (HashPasswordError),
        (SaveNewUserError),
    };

    operation HashPassword = (Password) -> { PasswordHash, HashPasswordError };

    error HashPasswordError = { };

    record NewUser = (email, password_hash, name);

    operation SaveNewUser = async (NewUser) -> { (), SaveNewUserError };

    error SaveNewUserError = {
        EmailTaken,
    };

    record LoginData = (maybe_email, maybe_password);

    operation Login = async (LoginData) -> { AuthToken, LoginError };

    error LoginError = {
        (FindUserError),
        (VerifyPasswordError),
        (IssueTokenError),
    };

    record DatabaseUser = (db_user_id, email, password_hash, name);

    operation FindUser = async (MaybeEmail) -> { DatabaseUser, FindUserError };

    error FindUserError = {
        NotFound
    };

    record PasswordClaim = (maybe_password, password_hash);

    operation VerifyPassword = (PasswordClaim) -> { (), VerifyPasswordError };

    error VerifyPasswordError = {
        InvalidPassword
    };

    operation IssueToken = (UserId) -> { AuthToken, IssueTokenError };

    error IssueTokenError = { };
}
