use std::io;

/// Error type for forwarding errors encountered while comparing two files.
pub enum Error {
    Lhs(io::Error),
    Rhs(io::Error),
}

/// Result of an operation performed while comparing two files.
pub type Result<T> = std::result::Result<T, Error>;

/// Trait providing methods for easy wrapping of `io::Error` into `Error`.
pub trait IoResult<T> {
    fn wrap_lhs(self) -> Result<T>;
    fn wrap_rhs(self) -> Result<T>;
}

impl<T> IoResult<T> for io::Result<T> {
    fn wrap_lhs(self) -> Result<T> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(Error::Lhs(e)),
        }
    }

    fn wrap_rhs(self) -> Result<T> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(Error::Rhs(e)),
        }
    }
}
