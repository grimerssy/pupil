use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::Deserialize;

use crate::{
    app::{performance::get_signature, AppContext},
    domain::performance::{KeyLookupError, SignedEvaluation},
    error::Error,
};

use super::{
    error::HttpError,
    middleware::template::{Template, TemplateName},
};

pub fn performance_routes() -> Router<AppContext> {
    Router::new().route("/{key}", get(student_evaluation))
}

#[derive(Clone, Debug, Deserialize)]
struct EvaluationPath {
    key: String,
}

async fn student_evaluation(
    State(ctx): State<AppContext>,
    Path(path): Path<EvaluationPath>,
) -> Result<Json<SignedEvaluation>, Template<Error<KeyLookupError>>> {
    get_signature(&ctx, path.key)
        .await
        .map(Json)
        .map_err(|error| Template::new(TemplateName::error(), error))
}

impl HttpError for KeyLookupError {
    fn status_code(&self) -> StatusCode {
        match self {
            KeyLookupError::UnknownKey => StatusCode::NOT_FOUND,
        }
    }
}
