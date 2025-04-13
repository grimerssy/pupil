#![allow(clippy::disallowed_types)]

use std::convert::Infallible;

use axum::{
    extract::{rejection::QueryRejection, FromRequestParts, Query as AxumQuery},
    http::request::Parts,
};
use serde::de::DeserializeOwned;

use crate::http::ExtractionError;

pub struct Query<T>(crate::Result<T>);

impl<T> Query<T> {
    pub fn consume(self) -> crate::Result<T> {
        self.0
    }
}

impl<T, S> FromRequestParts<S> for Query<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let value = AxumQuery::from_request_parts(parts, state)
            .await
            .map(|AxumQuery(value)| value)
            .map_err(ExtractionError::from)
            .map_err(crate::Error::from);
        Ok(Self(value))
    }
}

impl From<QueryRejection> for ExtractionError {
    fn from(value: QueryRejection) -> Self {
        Self {
            status: value.status(),
            body: value.body_text(),
        }
    }
}
