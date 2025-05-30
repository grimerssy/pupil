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
        auth::{login, signup},
        validation::{try_convert, ValidationErrors},
        AppContext, AppError,
    },
    domain::{
        auth::{LoginData, LoginError, SignupData, SignupError},
        token::AuthToken,
    },
    Error,
};

use super::{
    error::HttpError,
    middleware::{template::Template, view::View},
    serialize_secret,
};

const SIGNUP_PAGE: &str = "signup.html";

const LOGIN_PAGE: &str = "login.html";

const AUTH_TOKEN_SCRIPT: &str = "auth-token.html";

pub fn auth_routes() -> Router<AppContext> {
    let signup = Router::new()
        .route("/", get(singup_page))
        .route("/", post(handle_signup));
    let login = Router::new()
        .route("/", get(login_page))
        .route("/", post(handle_login));
    Router::new().nest("/signup", signup).nest("/login", login)
}

type HttpSignupError = Error<AppError<SignupError>, SignupForm>;

type HttpLoginError = Error<AppError<LoginError>, LoginForm>;

pub async fn singup_page() -> Template<()> {
    Template::new(SIGNUP_PAGE, ())
}

pub async fn login_page() -> Template<()> {
    Template::new(LOGIN_PAGE, ())
}

pub async fn handle_signup(
    State(ctx): State<AppContext>,
    Form(form): Form<SignupForm>,
) -> Result<Redirect, View<HttpSignupError>> {
    let form_copy = form.clone();
    signup(&ctx, form)
        .await
        .map(|_| Redirect::to("/"))
        .map_err(|error| View::new(SIGNUP_PAGE, error.with_input(form_copy)))
}

pub async fn handle_login(
    State(ctx): State<AppContext>,
    Form(form): Form<LoginForm>,
) -> Result<View<LoginResponse>, ErrorView<AppError<LoginError>>> {
    let form_copy = form.clone();
    login(&ctx, form)
        .await
        .map(|access_token| View::new(AUTH_TOKEN_SCRIPT, LoginResponse { access_token }))
        .map_err(|error| View::new(LOGIN_PAGE, error.with_input(form_copy)))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignupForm {
    email: String,
    #[serde(serialize_with = "serialize_secret")]
    password: SecretString,
    name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginForm {
    email: String,
    #[serde(serialize_with = "serialize_secret")]
    password: SecretString,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    access_token: AuthToken,
}

impl HttpError for SignupError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::EmailTaken => StatusCode::CONFLICT,
        }
    }
}

impl HttpError for LoginError {
    fn status_code(&self) -> StatusCode {
        StatusCode::UNAUTHORIZED
    }
}

impl TryFrom<SignupForm> for SignupData {
    type Error = ValidationErrors;

    fn try_from(value: SignupForm) -> Result<Self, Self::Error> {
        try_convert!(SignupForm value => SignupData {
            email,
            password,
            name
        })
    }
}

impl TryFrom<LoginForm> for LoginData {
    type Error = ValidationErrors;

    fn try_from(value: LoginForm) -> Result<Self, Self::Error> {
        Ok(Self {
            email: value.email.into(),
            password: value.password.into(),
        })
    }
}
