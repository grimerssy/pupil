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
}
