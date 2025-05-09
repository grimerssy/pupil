use axum::{
    extract::{Path, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use serde::Deserialize;
use unic_langid::LanguageIdentifier;

use crate::prelude::AppContext;

pub trait DefaultLanguage {
    fn default_language(&self) -> &LanguageIdentifier;
}

pub trait LookupLanguage {
    fn lookup_language(&self, language: &LanguageIdentifier) -> bool;
}

pub async fn redirect_to_default_locale(State(ctx): State<AppContext>) -> Redirect {
    let localized_root = format!("/{}/", ctx.default_language());
    Redirect::permanent(&localized_root)
}

#[derive(Debug, Deserialize)]
pub struct PathSegments {
    locale: String,
}

pub async fn assert_valid_locale(
    Path(PathSegments { locale }): Path<PathSegments>,
    State(ctx): State<AppContext>,
    req: Request,
    next: Next,
) -> Response {
    tracing::warn!(locale);
    if locale
        .parse::<LanguageIdentifier>()
        .is_ok_and(|lang| ctx.lookup_language(&lang))
    {
        next.run(req).await
    } else {
        (StatusCode::BAD_REQUEST, "Invalid locale segment in path").into_response()
    }
}
