pub(crate) use macros::*;

use std::{borrow::Cow, collections::HashMap};

use serde::{Deserialize, Serialize};

use crate::domain::error::DomainError;

use super::validation::ValidationErrors;

pub trait ContextualError {
    fn error_context(self) -> ErrorContext;
}

#[derive(Debug)]
pub enum AppError<E> {
    Validation(ValidationErrors),
    Logical(DomainError<E>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorContext {
    error_code: Cow<'static, str>,
    args: Option<HashMap<Cow<'static, str>, ErrorArgument>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ErrorArgument {
    Number(f64),
}

impl ErrorContext {
    pub const fn new(error_code: &'static str) -> Self {
        let error_code = Cow::Borrowed(error_code);
        Self {
            error_code,
            args: None,
        }
    }

    pub fn error_code(&self) -> &str {
        &self.error_code
    }

    pub fn args(&self) -> impl Iterator<Item = (&str, &ErrorArgument)> {
        self.args
            .iter()
            .flat_map(|args| args.iter())
            .map(|(key, value)| (key.as_ref(), value))
    }

    pub fn with_number(self, key: &'static str, value: impl Into<f64>) -> Self {
        self.with_arg(key, ErrorArgument::Number(value.into()))
    }

    fn with_arg(self, key: &'static str, arg: ErrorArgument) -> Self {
        let mut args = self.args.unwrap_or_default();
        args.insert(Cow::Borrowed(key), arg);
        Self {
            error_code: self.error_code,
            args: Some(args),
        }
    }
}

mod macros {
    macro_rules! log_error {
        ($error:expr) => {{
            use $crate::app::error::AppError as AE;
            use $crate::domain::error::DomainError as DE;
            match &$error {
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
