use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub enum Error {
    ReqwestError(reqwest::Error),
    ApiError(reqwest::StatusCode),
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
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Error::ReqwestError(error)
    }
}
