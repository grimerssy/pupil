use accept_header::Accept;
use axum::{
    extract::Request,
    http::header::ACCEPT,
    middleware::Next,
    response::{IntoResponse, Response},
    Extension, Json,
};
use mime::{APPLICATION_JSON, TEXT_HTML};

use crate::{
    app::error::AppError,
    http::response::{HttpResponse, ResponseContext},
};

use super::template::{Template, TemplateName};

pub type ErrorView<I, E> = View<AppError<I, E>>;

#[derive(Clone, Debug)]
pub struct View<T> {
    template_name: TemplateName,
    data: T,
}

impl<T> View<T> {
    pub fn new(template_name: impl Into<TemplateName>, data: T) -> Self {
        Self {
            template_name: template_name.into(),
            data,
        }
    }
}

pub(super) async fn render_view(req: Request, next: Next) -> Response {
    let accept_header = req.headers().get(ACCEPT).cloned();
    let mut response = next.run(req).await;
    let Some(view) = response.extensions_mut().remove::<View<HttpResponse>>() else {
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
        mime if mime == TEXT_HTML => Template::new(view.template_name, view.data).into_response(),
        _ => unreachable!(),
    };
    let (parts, _) = response.into_parts();
    (parts, body).into_response()
}

impl IntoResponse for View<HttpResponse> {
    fn into_response(self) -> Response {
        Extension(self).into_response()
    }
}

impl<T> IntoResponse for View<T>
where
    T: ResponseContext,
{
    fn into_response(self) -> Response {
        self.data
            .with_body(|response| View::new(self.template_name, response))
    }
}
