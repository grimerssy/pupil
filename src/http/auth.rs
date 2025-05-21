use axum::{
    extract::State,
    http::StatusCode,
    response::Redirect,
    routing::{get, post},
    Form, Router,
};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};

use crate::{
    app::{
        auth::signup,
        validation::{try_convert, ConversionFailure},
        AppContext,
    },
    domain::auth::{SaveNewUserError, SignupData, SignupError},
};

use super::{
    error::HttpError,
    middleware::{
        template::Template,
        view::{ErrorView, View},
    },
    response::Success,
    serialize_secret,
};

const SIGNUP_PAGE: &str = "signup.html";

pub fn auth_routes() -> Router<AppContext> {
    let signup = Router::new()
        .route("/", get(singup_page))
        .route("/", post(handle_signup));
    Router::new().nest("/signup", signup)
}

pub async fn singup_page() -> Template<Success<()>> {
    Template::new(SIGNUP_PAGE, Success(()))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignupForm {
    email: String,
    #[serde(serialize_with = "serialize_secret")]
    password: SecretString,
    name: String,
}

impl HttpError for SignupError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::HashPasswordError(error) => match *error {},
            Self::SaveNewUserError(error) => match error {
                SaveNewUserError::EmailTaken => StatusCode::CONFLICT,
            },
        }
    }
}

pub async fn handle_signup(
    State(ctx): State<AppContext>,
    Form(form): Form<SignupForm>,
) -> Result<Redirect, ErrorView<SignupForm, SignupError>> {
    signup(&ctx, form)
        .await
        .map(|_| Redirect::to("/"))
        .map_err(|error| View::new(SIGNUP_PAGE, error))
}

impl TryFrom<SignupForm> for SignupData {
    type Error = ConversionFailure<SignupForm>;

    fn try_from(value: SignupForm) -> Result<Self, Self::Error> {
        try_convert!(SignupForm value => SignupData {
            email,
            password,
            name
        })
    }
}
