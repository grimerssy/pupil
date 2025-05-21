use core::fmt;
use std::convert::Infallible;

use educe::Educe;

use crate::app::error::{ContextualError, ErrorContext};

pub type DomainResult<T, E> = core::result::Result<T, DomainError<E>>;

pub type InternalError = DomainError<Infallible>;

#[derive(Educe)]
#[educe(Debug)]
pub enum DomainError<E> {
    Expected(E),
    Internal(#[educe(Debug(method(fmt_error_chain)))] anyhow::Error),
}

impl<E> ContextualError for DomainError<E>
where
    E: ContextualError,
{
    fn error_context(self) -> ErrorContext {
        match self {
            Self::Internal(_) => ErrorContext::new("INTERNAL"),
            Self::Expected(error) => error.error_context(),
        }
    }
}

impl<E> DomainError<E> {
    pub fn cast<F>(self) -> DomainError<F>
    where
        F: From<E>,
    {
        match self {
            DomainError::Expected(domain) => DomainError::Expected(F::from(domain)),
            DomainError::Internal(internal) => DomainError::Internal(internal),
        }
    }
}

fn fmt_error_chain(error: &anyhow::Error, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{error}")?;
    core::iter::successors(error.source(), |src| src.source())
        .try_for_each(|src| write!(f, ": {src}"))
}

impl From<anyhow::Error> for InternalError {
    fn from(value: anyhow::Error) -> Self {
        DomainError::Internal(value)
    }
}

impl ContextualError for Infallible {
    fn error_context(self) -> ErrorContext {
        match self {}
    }
}
