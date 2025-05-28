use std::any::Any;

use crate::http::{middleware::template::TemplateName, response::Rejection};
use anyhow::anyhow;
use axum::response::{IntoResponse, Response};

use super::view::View;

#[tracing::instrument(skip(panic_message))]
pub fn catch_panic(panic_message: Box<dyn Any + Send + 'static>) -> Response {
    let error = if let Some(msg) = panic_message.downcast_ref::<String>() {
        anyhow!("{msg}")
    } else if let Some(msg) = panic_message.downcast_ref::<&str>() {
        anyhow!("{msg}")
    } else {
        anyhow!("unknown panic message")
    }
    .context("catch panic");
    let error = <crate::Error>::Internal(error);
    tracing::error!(?error);
    View::new(TemplateName::error(), Rejection(error)).into_response()
}
