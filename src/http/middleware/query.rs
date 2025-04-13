#![allow(clippy::disallowed_types)]

use std::convert::Infallible;

use axum::{
    extract::{FromRequestParts, Query as AxumQuery},
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
    T: DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let result = match AxumQuery::from_request_parts(parts, state).await {
            Ok(AxumQuery(value)) => Ok(value),
            Err(rejection) => Err(ExtractionError::from_rejection(rejection).await.into()),
        };
        Ok(Self(result))
    }
}
