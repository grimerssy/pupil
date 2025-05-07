use core::fmt;
use std::convert::Infallible;

use educe::Educe;

pub type DomainResult<T, E> = core::result::Result<T, DomainError<E>>;

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct InternalError(DomainError<Infallible>);

#[derive(Educe, thiserror::Error)]
#[educe(Debug)]
pub enum DomainError<E>
where
    E: std::error::Error,
{
    #[error(transparent)]
    Expected(E),
    #[error("An unexpected error occurred")]
    Internal(#[educe(Debug(method(fmt_error_chain)))] anyhow::Error),
}

impl<E> DomainError<E>
where
    E: std::error::Error,
{
    pub fn cast<F>(self) -> DomainError<F>
    where
        F: std::error::Error + From<E>,
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
        Self(DomainError::Internal(value))
    }
}
