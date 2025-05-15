use accept_header::Accept;
use axum::{
    extract::Request,
    http::header::ACCEPT,
    middleware::Next,
    response::{IntoResponse, Response},
    Extension, Json,
};
use mime::{APPLICATION_JSON, TEXT_HTML};
use serde::Serialize;

use crate::http::{
    error::HttpError,
    response::{HttpResponse, HttpResponseExtension},
};

use super::template::{Template, TemplateMeta};

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

pub(super) async fn render_view(req: Request, next: Next) -> Response {
    let accept_header = req.headers().get(ACCEPT).cloned();
    let mut response = next.run(req).await;
    let Some(view) = response
        .extensions_mut()
        .remove::<View<HttpResponseExtension>>()
    else {
        return response;
    };
    let preference = accept_header
        .and_then(|header| {
            header
                .to_str()
                .ok()
                .and_then(|header| header.parse::<Accept>().ok())
        })
        .and_then(|accept| accept.negotiate(&[APPLICATION_JSON, TEXT_HTML]).ok())
        .unwrap_or(TEXT_HTML);
    let body = match preference {
        mime if mime == APPLICATION_JSON => Json(view.data).into_response(),
        mime if mime == TEXT_HTML => {
            Template::with_meta(view.template_meta, view.data).into_response()
        }
        _ => unreachable!(),
    };
    let (parts, _) = response.into_parts();
    (parts, body).into_response()
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
