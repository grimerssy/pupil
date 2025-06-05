use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
    Router,
};
use serde::Deserialize;

use crate::{
    app::{keys::remove_key, AppContext},
    domain::{
        auth::User,
        key::Key,
        keys::{GenerateKey, GenerateKeyError, GetKeys, RemoveKeyError},
    },
    error::Error,
};

use super::{
    error::HttpError,
    middleware::template::{Template, TemplateName},
};

const KEYS: &str = "components/keys.html";

pub fn keys_routes() -> Router<AppContext> {
    Router::new()
        .route("/", get(my_keys))
        .route("/gen", post(generate_key))
        .route("/{key}", delete(delete_key))
}

async fn my_keys(
    user: User,
    State(ctx): State<AppContext>,
) -> Result<Template<Vec<Key>>, Template<Error>> {
    ctx.get_keys(user.id)
        .await
        .map(|keys| Template::new(KEYS, keys))
        .map_err(|error| Template::new(TemplateName::error(), error))
}

async fn generate_key(
    user: User,
    State(ctx): State<AppContext>,
) -> Result<Template<Vec<Key>>, Template<Error<GenerateKeyError>>> {
    ctx.generate_key(user.id)
        .await
        .map(|keys| Template::new(KEYS, keys))
        .map_err(|error| Template::new(TemplateName::error(), error))
}

#[derive(Clone, Debug, Deserialize)]
struct KeyPath {
    key: String,
}

async fn delete_key(
    user: User,
    State(ctx): State<AppContext>,
    Path(path): Path<KeyPath>,
) -> Result<Template<Vec<Key>>, Template<Error<RemoveKeyError>>> {
    remove_key(&ctx, user.id, path.key)
        .await
        .map(|keys| Template::new(KEYS, keys))
        .map_err(|error| Template::new(TemplateName::error(), error))
}

impl HttpError for GenerateKeyError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::UnknownUser => StatusCode::FORBIDDEN,
        }
    }
}

impl HttpError for RemoveKeyError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::UnknownKey => StatusCode::NOT_FOUND,
        }
    }
}
