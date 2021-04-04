use std::fmt::{self, Display, Formatter};

/// Type that describes WorkerClient errors
#[derive(Debug)]
pub enum Error {
    /// Problems found by the `Reqwest` client.
    ReqwestError(reqwest::Error),
    /// HTTP error codes from the API server.
    ApiError(reqwest::StatusCode),
    /// Errors related to reading files (certificate).
    FileError(std::io::Error),
    /// Errors on internal work of the `WorkerClient`.
    InternalError,
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::ReqwestError(err) => write!(f, "Reqwest error: {}", err),
            Error::ApiError(status) => write!(
                f,
                "API error: {}",
                status.canonical_reason().unwrap_or("unknown")
            ),
            Error::FileError(err) => write!(f, "File error: {}", err),
            Error::InternalError => write!(f, "Internal error"),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Error::ReqwestError(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::FileError(error)
    }
}
