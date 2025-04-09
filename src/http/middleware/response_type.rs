use std::convert::Infallible;

use axum::{
    extract::FromRequestParts,
    http::{header::ACCEPT, request::Parts, HeaderValue},
};

#[derive(Clone, Debug, Default)]
pub struct LazyResponseType {
    accept: Option<HeaderValue>,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum ResponseType {
    #[default]
    Html,
    Json,
}

impl<S> FromRequestParts<S> for LazyResponseType
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let accept = parts.headers.get(ACCEPT).cloned();
        Ok(Self { accept })
    }
}

impl LazyResponseType {
    pub fn parse(self) -> ResponseType {
        match self.accept {
            Some(accept) if accept.as_ref().starts_with(b"application/json") => ResponseType::Json,
            _ => ResponseType::default(),
        }
    }
}

impl<S> FromRequestParts<S> for ResponseType
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        LazyResponseType::from_request_parts(parts, state)
            .await
            .map(LazyResponseType::parse)
    }
}
