use core::fmt;
use std::convert::Infallible;

use educe::Educe;

pub type Result<T, E = Infallible> = core::result::Result<T, Error<E>>;

#[derive(Educe)]
#[educe(Debug)]
pub enum Error<E = Infallible> {
    Expected(E),
    Internal(#[educe(Debug(method(fmt_error_chain)))] anyhow::Error),
}

pub struct Rejection<I, E> {
    pub input: I,
    pub error: E,
}

impl<E> Error<E> {
    pub fn cast<F>(self) -> Error<F>
    where
        E: Into<F>
    {
        match self {
            Self::Expected(error) => Error::Expected(error.into()),
            Self::Internal(internal) => Error::Internal(internal),
        }
    }

    pub fn from_internal(error: Error) -> Self {
        match error {
            Error::Expected(never) => match never {},
            Error::Internal(error) => Self::Internal(error),
        }
    }
}

impl<E> From<anyhow::Error> for Error<E> {
    fn from(value: anyhow::Error) -> Self {
        Self::Internal(value)
    }
}

fn fmt_error_chain(error: &anyhow::Error, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{error}")?;
    core::iter::successors(error.source(), |src| src.source())
        .try_for_each(|src| write!(f, ": {src}"))
}
