use std::{borrow::Cow, collections::HashMap, convert::Infallible};

use serde::{Deserialize, Serialize};

use crate::error::ErrorKind;

use super::validation::ValidationErrors;

pub trait ContextualError {
    fn error_context(self) -> ErrorContext;
}

#[derive(Debug)]
pub enum AppError<E> {
    Validation(ValidationErrors),
    Logical(E),
}

impl<E> From<E> for AppError<E> {
    fn from(value: E) -> Self {
        Self::Logical(value)
    }
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

impl ContextualError for Infallible {
    fn error_context(self) -> ErrorContext {
        match self {}
    }
}

impl<E> ContextualError for crate::Error<E>
where
    E: ContextualError,
{
    fn error_context(self) -> ErrorContext {
        match self.kind {
            ErrorKind::Internal(_) => ErrorContext::new("INTERNAL"),
            ErrorKind::Expected(error) => error.error_context(),
        }
    }
}
