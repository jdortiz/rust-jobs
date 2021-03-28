use std::{
    error,
    fmt::{self, Display},
};

// Custom error type for Jobs.
#[derive(Debug)]
pub enum JobError {
    InvalidCommand(String),
}

impl error::Error for JobError {}

impl Display for JobError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid Job")
    }
}
