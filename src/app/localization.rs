use std::{borrow::Cow, collections::HashMap, convert::Infallible};

use serde::{Deserialize, Serialize};

use crate::error::ErrorKind;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalizedError {
    error_code: Cow<'static, str>,
    args: Option<HashMap<Cow<'static, str>, Argument>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Argument {
    Number(f64),
}

impl LocalizedError {
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

    pub fn args(&self) -> impl Iterator<Item = (&str, &Argument)> {
        self.args
            .iter()
            .flat_map(|args| args.iter())
            .map(|(key, value)| (key.as_ref(), value))
    }

    pub fn with_number(self, key: &'static str, value: impl Into<f64>) -> Self {
        self.with_arg(key, Argument::Number(value.into()))
    }

    fn with_arg(self, key: &'static str, arg: Argument) -> Self {
        let mut args = self.args.unwrap_or_default();
        args.insert(Cow::Borrowed(key), arg);
        Self {
            error_code: self.error_code,
            args: Some(args),
        }
    }
}

impl From<Infallible> for LocalizedError {
    fn from(value: Infallible) -> Self {
        match value {}
    }
}

impl<E> From<ErrorKind<E>> for LocalizedError
where
    E: Into<Self>,
{
    fn from(value: ErrorKind<E>) -> Self {
        match value {
            ErrorKind::Internal(_) => LocalizedError::new("INTERNAL"),
            ErrorKind::Expected(error) => error.into(),
        }
    }
}
