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
        write!(f, "{self}")?;
        core::iter::successors(self.0.source(), |src| src.source())
            .try_for_each(|src| write!(f, ": {src}"))
    }
}
