use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
};

use crate::{
    app::{auth::authenticate, localization::LocalizedError, AppContext},
    domain::{auth::User, token::AuthToken},
    error::Error,
    http::{error::HttpError, middleware::template::TemplateName},
};

use super::view::View;

#[derive(Debug)]
pub struct Unauthorized;

impl FromRequestParts<AppContext> for User {
    type Rejection = View<Error<Unauthorized>>;

    async fn from_request_parts(
        parts: &mut Parts,
        ctx: &AppContext,
    ) -> Result<Self, Self::Rejection> {
        let parse_bearer: fn(&str) -> Option<&str> = |auth: &str| match auth.split_once(' ') {
            Some(("Bearer", token)) => Some(token),
            _ => None,
        };
        let token = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|header| header.to_str().ok().and_then(parse_bearer))
            .map(|value| AuthToken::new(value.to_owned()));
        let user = match token {
            Some(token) => authenticate(ctx, token).await.ok(),
            None => None,
        };
        user.ok_or(Error::expected(Unauthorized))
            .inspect_err(|error| tracing::info!(?error))
            .map_err(|error| View::new(TemplateName::error(), error))
    }
}

impl HttpError for Unauthorized {
    fn status_code(&self) -> StatusCode {
        StatusCode::FORBIDDEN
    }
}

impl From<Unauthorized> for LocalizedError {
    fn from(_: Unauthorized) -> Self {
        Self::new("UNAUTHORIZED")
    }
}
