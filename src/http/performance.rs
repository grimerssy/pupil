use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Router,
};
use serde::Deserialize;

use crate::{
    app::{performance::get_signature, AppContext},
    domain::{
        performance::{GetVerifyingKey, KeyLookupError, SignedEvaluation},
        verifying_key::VerifyingKey,
    },
    error::Error,
};

use super::{error::HttpError, middleware::json::Json};

pub fn performance_routes() -> Router<AppContext> {
    Router::new()
        .route("/verifying-key", get(verifying_key))
        .route("/{key}", get(student_evaluation))
}

#[derive(Clone, Debug, Deserialize)]
struct EvaluationPath {
    key: String,
}

async fn verifying_key(State(ctx): State<AppContext>) -> Result<Json<VerifyingKey>, Json<Error>> {
    ctx.get_verifying_key().map(Json).map_err(Json)
}

async fn student_evaluation(
    State(ctx): State<AppContext>,
    Path(path): Path<EvaluationPath>,
) -> Result<Json<SignedEvaluation>, Json<Error<KeyLookupError>>> {
    get_signature(&ctx, path.key).await.map(Json).map_err(Json)
}

impl HttpError for KeyLookupError {
    fn status_code(&self) -> StatusCode {
        match self {
            KeyLookupError::UnknownKey => StatusCode::NOT_FOUND,
        }
    }
}
