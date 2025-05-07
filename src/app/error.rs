use educe::Educe;
pub(crate) use macros::*;

use crate::domain::error::DomainError;

use super::validation::{ConversionFailure, ValidationErrors};

pub type AppResult<I, O, E> = Result<O, AppError<I, E>>;

#[derive(Educe, thiserror::Error)]
#[educe(Debug)]
#[error("{kind}")]
pub struct AppError<I, E>
where
    E: std::error::Error,
{
    #[educe(Debug(ignore))]
    pub input: I,
    pub kind: AppErrorKind<E>,
}

#[derive(Debug, thiserror::Error)]
pub enum AppErrorKind<E>
where
    E: std::error::Error,
{
    #[error(transparent)]
    Validation(ValidationErrors),
    #[error(transparent)]
    Logical(DomainError<E>),
}

impl<E> AppErrorKind<E>
where
    E: std::error::Error,
{
    pub fn with_input<I>(self, input: I) -> AppError<I, E> {
        AppError { input, kind: self }
    }
}

impl<T, E> From<ConversionFailure<T>> for AppError<T, E>
where
    E: std::error::Error,
{
    fn from(value: ConversionFailure<T>) -> Self {
        let ConversionFailure { input, errors } = value;
        let kind = AppErrorKind::Validation(errors);
        Self { input, kind }
    }
}

mod macros {
    macro_rules! log_error {
        ($error:expr) => {{
            use $crate::app::error::AppErrorKind as AE;
            use $crate::domain::error::DomainError as DE;
            match &$error.kind {
                AE::Validation(errors) => ::tracing::info!(?errors),
                AE::Logical(DE::Expected(error)) => ::tracing::info!(?error),
                AE::Logical(DE::Internal(error)) => ::tracing::error!(?error),
            }
        }};
    }

    macro_rules! log_result {
        (async $result:block) => {{
            #[allow(clippy::redundant_closure_call)]
            let result = (async || { $result })().await;
            $crate::app::error::log_result!(result)
        }};
        ($result:block) => {{
            #[allow(clippy::redundant_closure_call)]
            let result = (|| { $result })();
            $crate::app::error::log_result!(result)
        }};
        ($result:expr) => {{
            let result = $result;
            match &result {
                Ok(success) => ::tracing::info!(return = ?success),
                Err(error) => {
                    $crate::app::error::log_error!(error);
                },
            }
            result
        }};
    }

    pub(crate) use {log_error, log_result};
}
