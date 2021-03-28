use crate::{JobError, JobStatus};
use std::{fs::File, process::Stdio};
use tokio::process::{Child, Command};
use uuid::Uuid;

/// Type that contains the details of a job.
pub struct Job {
    id: Uuid,
    command_line: String,
    child: Option<Child>,
    status: JobStatus,
}

impl Job {
    /// Creates a new `Job` with the given command line and a new
    /// UUID.  It spawns the associated command right away.
    ///
    /// * `id` - UUID that will be assigned to the `Job`.
    /// * `command_line` - Command line that will be executed in this job.
    pub fn new(id: Uuid, command_line: &str) -> Result<Job, JobError> {
        let job = Job {
            id,
            command_line: command_line.to_string(),
            child: None,
            status: JobStatus::InProgress,
        };
        job.start()
    }

    // Start the job in a different process. This is a private
    // function because `Job`s are immediatelly started from `new()`
    fn start(mut self) -> Result<Job, JobError> {
        // TODO: This doesn't take into account quotes.
        let mut parts = self.command_line.split_whitespace();
        let command = parts
            .next()
            .ok_or(JobError::InvalidCommand(self.command_line.to_string()))?;
        let args = parts;
        let filename = format!("{}.txt", self.id);
        let output = File::create(filename)?;
        let error = output.try_clone()?;
        self.child = Some(
            Command::new(command)
                .args(args)
                .stdout(Stdio::from(output))
                .stderr(Stdio::from(error))
                .spawn()?,
        );

        Ok(self)
    }

    /// Return the status of job.
    pub fn status(&mut self) -> JobStatus {
        // Refresh status only if InProgress
        if matches!(self.status, JobStatus::InProgress) {
            if let Some(ref mut child) = self.child {
                match child.try_wait() {
                    Ok(None) => {}
                    Ok(Some(status)) => {
                        self.status = JobStatus::Done(status);
                        self.child = None;
                    }
                    Err(_) => {
                        self.status = JobStatus::Failed;
                        self.child = None;
                    }
                }
            }
        }

        self.status.clone()
    }

    /// Stop the job using a kill signal.
    pub fn stop(&mut self) {
        if matches!(self.status, JobStatus::InProgress) {
            if let Some(ref mut child) = self.child {
                match child.start_kill() {
                    Ok(_) => {} //self.status = JobStatus::Stopped,
                    Err(_) => {
                        self.status = JobStatus::Failed;
                        self.child = None;
                    }
                }
            }
        }
    }

    /// Get the value of the job id. This is a uuid.
    pub fn get_id(&self) -> Uuid {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use super::*;

    #[tokio::test]
    async fn new_produces_valid_job() {
        let job = Job::new(Uuid::new_v4(), "ls");

        assert!(job.is_ok())
    }

    #[tokio::test]
    async fn new_preserves_id() {
        let id = Uuid::new_v4();
        let job = Job::new(id, "ls").unwrap();

        assert_eq!(id, job.get_id());
    }

    #[tokio::test]
    async fn new_produces_error_if_command_is_empty() {
        let command = "  ";
        let job = Job::new(Uuid::new_v4(), command);

        assert!(job.is_err());
        assert!(
            matches!(job.err(), Some(JobError::InvalidCommand(actual_command)) if actual_command == command)
        );
    }

    #[tokio::test]
    async fn non_existing_command_returns_failure() {
        let job = Job::new(Uuid::new_v4(), "mxyzptlk -s");

        assert!(job.is_err());
        assert!(matches!(job.err(), Some(JobError::CommandNotFound)));
    }

    #[tokio::test]
    async fn valid_command_initial_status_is_in_progress() {
        let mut job = Job::new(Uuid::new_v4(), "sleep 1").unwrap();

        assert!(matches!(job.status(), JobStatus::InProgress));
    }

    #[tokio::test]
    async fn valid_command_status_is_done_successful() {
        let mut job = Job::new(Uuid::new_v4(), "true").unwrap();

        while matches!(job.status(), JobStatus::InProgress) {
            thread::sleep(Duration::from_millis(50));
        }

        assert!(matches!(job.status(), JobStatus::Done(ref status) if status.success()));
    }

    #[tokio::test]
    async fn failing_command_status_is_done_failed() {
        let mut job = Job::new(Uuid::new_v4(), "false").unwrap();

        while matches!(job.status(), JobStatus::InProgress) {
            thread::sleep(Duration::from_millis(50));
        }

        assert!(matches!(job.status(), JobStatus::Done(ref status) if !status.success()));
    }

    #[tokio::test]
    async fn long_running_command_can_be_stopped() {
        let mut job = Job::new(Uuid::new_v4(), "sleep 100").unwrap();

        let mut i = 0;
        while matches!(job.status(), JobStatus::InProgress) {
            thread::sleep(Duration::from_millis(50));
            if i == 7 {
                job.stop()
            }
            i += 1;
        }

        assert!(i < 10);
        assert!(matches!(job.status(), JobStatus::Done(ref status) if !status.success()));
    }
}
