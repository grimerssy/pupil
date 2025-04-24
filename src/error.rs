use core::fmt;

#[derive(Debug, thiserror::Error)]
pub enum Error<E>
where
    E: std::error::Error,
{
    #[error(transparent)]
    Domain(E),
    #[error("An unexpected error occurred")]
    Unexpected(InternalError),
}

#[derive(thiserror::Error)]
#[error(transparent)]
pub struct InternalError(#[from] anyhow::Error);

impl fmt::Debug for InternalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("InternalError")
            .field(&ErrorChain(&self.0))
            .finish()
    }
}

struct ErrorChain<'a>(&'a anyhow::Error);

impl fmt::Debug for ErrorChain<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let error = self.0;
        write!(f, "{error}")?;
        core::iter::successors(error.source(), |src| src.source())
            .try_for_each(|src| write!(f, ": {src}"))
    }
}
