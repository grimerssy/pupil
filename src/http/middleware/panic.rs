use std::any::Any;

use crate::http::middleware::template::TemplateName;
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
    let panic = <crate::Error>::internal(error);
    tracing::error!(?panic);
    View::new(TemplateName::error(), panic).into_response()
}
