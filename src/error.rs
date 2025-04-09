use core::fmt;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("An unexpected error occurred")]
    Internal(ErrorChain),
}

impl From<anyhow::Error> for Error {
    fn from(error: anyhow::Error) -> Self {
        Self::Internal(ErrorChain(error))
    }
}

#[derive(thiserror::Error)]
#[error(transparent)]
pub struct ErrorChain(#[from] anyhow::Error);

impl fmt::Debug for ErrorChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")?;
        std::iter::successors(self.0.source(), |err| err.source())
            .try_for_each(|err| write!(f, ": {err}"))
    }
}
