use crate::JobError;
use std::{
    error,
    fs::File,
    process::{Command, Stdio},
};
use uuid::Uuid;

/// Type that contains the details of a job.
pub struct Job {
    id: Uuid,
    command_line: String,
}

impl Job {
    /// Creates a new `Job` with the given command line and a new UUID.
    pub fn new(id: Uuid, command_line: &str) -> Result<Job, JobError> {
        let job = Job {
            id,
            command_line: command_line.to_string(),
        };
        match job.start() {
            Err(_) => Err(JobError::InvalidCommand(command_line.to_string())),
            Ok(_) => Ok(job),
        }
    }

    // Start the job in a different process. This is a private
    // function because `Job`s are immediatelly started from `new()`
    fn start(&self) -> Result<(), Box<dyn error::Error>> {
        // TODO: This doesn't take into account quotes.
        let mut parts = self.command_line.split_whitespace();
        let command = parts
            .next()
            .ok_or(JobError::InvalidCommand("".to_string()))?;
        let args = parts;
        let filename = format!("{}.txt", self.id);
        let output = File::create(filename)?;
        let error = output.try_clone()?;
        let child = Command::new(command)
            .args(args)
            .stdout(Stdio::from(output))
            .stderr(Stdio::from(error))
            .spawn()?;
        child.wait_with_output()?;

        Ok(())
    }

    /// Get the value of the job id. This is a uuid.
    pub fn get_id(&self) -> Uuid {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_produces_valid_job() {
        let job = Job::new(Uuid::new_v4(), "ls");

        assert!(job.is_ok())
    }

    #[test]
    fn new_preserves_id() {
        let id = Uuid::new_v4();
        let job = Job::new(id, "ls").unwrap();

        assert_eq!(id, job.get_id());
    }

    #[test]
    fn new_produces_error_if_command_is_empty() {
        let command = "  ";
        let job = Job::new(Uuid::new_v4(), command);

        assert!(job.is_err());
        assert!(
            matches!(job.err(), Some(JobError::InvalidCommand(actual_command)) if actual_command == command)
        );
    }

    #[test]
    fn non_existing_command_returns_failure() {
        let job = Job::new(Uuid::new_v4(), "mxyzptlk -s");

        assert!(job.is_err());
    }
}
