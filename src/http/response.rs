use private::Never;
use serde::Serialize;

use crate::app::validation::ValidationErrors;

pub type SuccessHttpResponse<T> = HttpResponse<Never, T, Never>;

pub type ErrorHttpResponse<I> = HttpResponse<I, Never, ValidationErrors>;

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum HttpResponse<I, O, V> {
    Success { data: O },
    Fail { input: I, data: V },
    Error { input: I, message: String },
}

pub type HttpResponseExtension = HttpResponse<OpaqueData, OpaqueData, OpaqueData>;

#[derive(Serialize)]
pub struct OpaqueData(Box<dyn erased_serde::Serialize + Send + Sync>);

impl<I, O, V> HttpResponse<I, O, V>
where
    I: Serialize + Send + Sync + 'static,
    O: Serialize + Send + Sync + 'static,
    V: Serialize + Send + Sync + 'static,
{
    pub fn erase_types(self) -> HttpResponseExtension {
        match self {
            Self::Success { data } => HttpResponseExtension::Success {
                data: OpaqueData(Box::new(data)),
            },
            Self::Fail { input, data } => HttpResponseExtension::Fail {
                input: OpaqueData(Box::new(input)),
                data: OpaqueData(Box::new(data)),
            },
            Self::Error { input, message } => HttpResponseExtension::Error {
                input: OpaqueData(Box::new(input)),
                message,
            },
        }
    }
}

impl<T> SuccessHttpResponse<T>
where
    T: Serialize,
{
    pub fn success(data: T) -> Self {
        Self::Success { data }
    }
}

impl Clone for HttpResponseExtension {
    fn clone(&self) -> Self {
        unreachable!("HTTP response extension may not be cloned")
    }
}

mod private {
    #[derive(serde::Serialize)]
    pub enum Never {}
}
