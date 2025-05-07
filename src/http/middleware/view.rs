use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
    Extension, Json,
};
use serde::Serialize;

use crate::http::{
    error::HttpError,
    response::{HttpResponse, HttpResponseExtension},
};

use super::{
    response_type::{LazyResponseType, ResponseType},
    template::{Template, TemplateMeta},
};

#[derive(Clone, Debug)]
pub struct View<T> {
    template_meta: TemplateMeta,
    data: T,
}

impl<T> View<T> {
    pub fn error(error: T) -> Self {
        let template_meta = TemplateMeta::error();
        Self::with_meta(template_meta, error)
    }

    fn with_meta(template_meta: TemplateMeta, data: T) -> Self {
        Self {
            template_meta,
            data,
        }
    }
}

pub(super) async fn handle_render_view(
    response_type: LazyResponseType,
    req: Request,
    next: Next,
) -> Response {
    let mut response = next.run(req).await;
    let Some(view) = response
        .extensions_mut()
        .remove::<View<HttpResponseExtension>>()
    else {
        return response;
    };
    let (parts, _) = response.into_parts();
    match response_type.parse() {
        ResponseType::Json => (parts, Json(view.data)).into_response(),
        ResponseType::Html => {
            (parts, Template::with_meta(view.template_meta, view.data)).into_response()
        }
    }
}

impl<I, O, V> IntoResponse for View<HttpResponse<I, O, V>>
where
    I: Serialize + Send + Sync + 'static,
    O: Serialize + Send + Sync + 'static,
    V: Serialize + Send + Sync + 'static,
{
    fn into_response(self) -> Response {
        let view = View::with_meta(self.template_meta, self.data.erase_types());
        Extension(view).into_response()
    }
}

impl<E> IntoResponse for View<E>
where
    E: HttpError,
{
    fn into_response(self) -> Response {
        self.data
            .with_body(|response| View::with_meta(self.template_meta, response))
    }
}
