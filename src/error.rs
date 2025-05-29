use core::fmt;
use std::convert::Infallible;

use educe::Educe;

pub type Result<T, E = Infallible, I = ()> = core::result::Result<T, Error<E, I>>;

#[derive(Educe)]
#[educe(Debug(named_field = false))]
pub struct Error<E = Infallible, I = ()> {
    #[educe(Debug(ignore))]
    pub input: I,
    pub kind: ErrorKind<E>,
}

#[derive(Educe)]
#[educe(Debug)]
pub enum ErrorKind<E = Infallible> {
    Expected(E),
    Internal(#[educe(Debug(method(fmt_error_chain)))] anyhow::Error),
}

impl<E> Error<E> {
    pub fn expected(error: E) -> Self {
        let kind = ErrorKind::Expected(error);
        Error { input: (), kind }
    }

    pub fn internal(error: anyhow::Error) -> Error<E> {
        let kind = ErrorKind::Internal(error);
        Error { input: (), kind }
    }

    pub fn with_input<I>(self, input: I) -> Error<E, I> {
        let kind = self.kind;
        Error { input, kind }
    }
}

impl<E, I> Error<E, I> {
    pub fn cast<F>(self) -> Error<F, I>
    where
        E: Into<F>,
    {
        let input = self.input;
        let kind = match self.kind {
            ErrorKind::Expected(error) => ErrorKind::Expected(error.into()),
            ErrorKind::Internal(internal) => ErrorKind::Internal(internal),
        };
        Error { input, kind }
    }

    pub fn from_internal(error: Error<Infallible, I>) -> Self {
        let input = error.input;
        let ErrorKind::Internal(internal_error) = error.kind;
        let kind = ErrorKind::Internal(internal_error);
        Self { input, kind }
    }
}

impl<E> From<anyhow::Error> for Error<E> {
    fn from(value: anyhow::Error) -> Self {
        Self::internal(value)
    }
}

fn fmt_error_chain(error: &anyhow::Error, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{error}")?;
    core::iter::successors(error.source(), |src| src.source())
        .try_for_each(|src| write!(f, ": {src}"))
}
