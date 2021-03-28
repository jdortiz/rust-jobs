use std::{
    error,
    fmt::{self, Display},
};

/// Custom error type for Jobs.
#[derive(Debug)]
pub enum JobError {
    /// Job command is invalid and cannot be run.  An empty string might be an example of this.
    InvalidCommand(String),
    /// The specified command cannot be found.
    CommandNotFound,
    /// There has been an I/O error when trying to run the command. An
    /// example of this could be not having permission to create a
    /// file to store the output temporary.  There might be several
    /// reasons for this, so you can check the embedded error.
    IoError(std::io::Error),
}

impl error::Error for JobError {}

impl Display for JobError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            JobError::CommandNotFound => write!(f, "Command not found"),
            JobError::IoError(ref err) => write!(f, "I/O error: {}", err),
            JobError::InvalidCommand(ref cmd) => write!(f, "Invalid command {}", cmd),
        }
    }
}

impl From<std::io::Error> for JobError {
    fn from(error: std::io::Error) -> Self {
        match error.kind() {
            std::io::ErrorKind::NotFound => JobError::CommandNotFound,
            _ => JobError::IoError(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::ErrorKind;

    use super::*;

    #[test]
    fn not_found_is_converted_to_command_not_found() {
        let err = JobError::from(std::io::Error::new(ErrorKind::NotFound, "not there!"));

        assert!(matches!(err, JobError::CommandNotFound));
    }

    #[test]
    fn other_io_errors_are_wrapped() {
        let err = JobError::from(std::io::Error::new(
            ErrorKind::PermissionDenied,
            "not there!",
        ));

        assert!(
            matches!(err, JobError::IoError(inner_error) if inner_error.kind() == ErrorKind::PermissionDenied)
        );
    }
}
