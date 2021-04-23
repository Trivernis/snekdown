use charred::error::TapeError;
use thiserror::Error;

pub type SnekdownResult<T> = Result<T, SnekdownError>;

#[derive(Debug, Error)]
pub enum SnekdownError {
    #[error(transparent)]
    TapeError(#[from] TapeError),
}
