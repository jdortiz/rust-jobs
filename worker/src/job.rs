use crate::{JobError, JobStatus};
use std::{fs::File, process::Stdio};
use tokio::process::{Child, Command};
use uuid::Uuid;

/// Type that contains the details of a job.
pub struct Job {
    id: Uuid,
    command_line: String,
    owner: String,
    child: Option<Child>,
    status: JobStatus,
}

impl Job {
    /// Creates a new `Job` with the given command line and a new
    /// UUID.  It spawns the associated command right away.
    ///
    /// * `id` - UUID that will be assigned to the `Job`.
    /// * `owner` - String id of the owner of the job.  It is used for authorizing operations.
    /// * `command_line` - Command line that will be executed in this job.
    pub fn new(id: Uuid, owner: &str, command_line: &str) -> Result<Job, JobError> {
        let job = Job {
            id,
            command_line: command_line.to_string(),
            owner: owner.to_string(),
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
            .ok_or_else(|| JobError::InvalidCommand(self.command_line.to_string()))?;
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
    ///
    /// * `as_user` - Perform this operation for this user id.  It
    /// must match the onwer or it will return a `Unauthorized` error.
    pub fn status(&mut self, as_user: &str) -> Result<JobStatus, JobError> {
        if as_user != self.owner {
            return Err(JobError::Unauthorized);
        }
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

        Ok(self.status.clone())
    }

    /// Return the filename that contains the output of the job.
    ///
    /// * `as_user` - Perform this operation for this user id.  It
    /// must match the onwer or it will return a `Unauthorized` error.
    pub fn output(&mut self, as_user: &str) -> Result<String, JobError> {
        if as_user != self.owner {
            return Err(JobError::Unauthorized);
        }
        Ok(format!("{}.txt", self.id))
    }

    /// Stop the job using a kill signal.
    ///
    /// * `as_user` - Perform this operation for this user id.  It
    /// must match the onwer or it will return a `Unauthorized` error.
    pub fn stop(&mut self, as_user: &str) -> Result<(), JobError> {
        if as_user != self.owner {
            return Err(JobError::Unauthorized);
        }
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
        Ok(())
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

    const OWNER_1: &str = "owner 1";
    const OWNER_2: &str = "owner 2";

    #[tokio::test]
    async fn new_produces_valid_job() {
        let job = Job::new(Uuid::new_v4(), OWNER_1, "ls");

        assert!(job.is_ok())
    }

    #[tokio::test]
    async fn new_preserves_id() {
        let id = Uuid::new_v4();
        let job = Job::new(id, OWNER_1, "ls").unwrap();

        assert_eq!(id, job.get_id());
    }

    #[tokio::test]
    async fn new_produces_error_if_command_is_empty() {
        let command = "  ";
        let job = Job::new(Uuid::new_v4(), OWNER_1, command);

        assert!(job.is_err());
        assert!(
            matches!(job.err(), Some(JobError::InvalidCommand(actual_command)) if actual_command == command)
        );
    }

    #[tokio::test]
    async fn non_existing_command_returns_failure() {
        let job = Job::new(Uuid::new_v4(), OWNER_1, "mxyzptlk -s");

        assert!(job.is_err());
        assert!(matches!(job.err(), Some(JobError::CommandNotFound)));
    }

    #[tokio::test]
    async fn command_status_is_only_available_to_owner() {
        let mut job = Job::new(Uuid::new_v4(), OWNER_1, "true").unwrap();

        assert!(job.status(OWNER_2).is_err());
        assert!(matches!(
            job.status(OWNER_2).err(),
            Some(JobError::Unauthorized)
        ));
    }

    #[tokio::test]
    async fn valid_command_initial_status_is_in_progress() {
        let mut job = Job::new(Uuid::new_v4(), OWNER_1, "sleep 1").unwrap();

        assert!(matches!(
            job.status(OWNER_1).ok(),
            Some(JobStatus::InProgress)
        ));
    }

    #[tokio::test]
    async fn command_output_filename_is_only_available_to_owner() {
        let mut job = Job::new(Uuid::new_v4(), OWNER_1, "true").unwrap();

        assert!(job.output(OWNER_2).is_err());
        assert!(matches!(
            job.output(OWNER_2).err(),
            Some(JobError::Unauthorized)
        ));
    }

    #[tokio::test]
    async fn owner_can_retrieve_output_filename() {
        let id = Uuid::new_v4();
        let mut job = Job::new(id, OWNER_1, "ls").unwrap();

        let filename = format!("{}.txt", id);
        assert!(matches!(job.output(OWNER_1), Ok(output) if output == filename));
    }

    #[tokio::test]
    async fn valid_command_status_is_done_successful() {
        let mut job = Job::new(Uuid::new_v4(), OWNER_1, "true").unwrap();

        while matches!(job.status(OWNER_1).ok(), Some(JobStatus::InProgress)) {
            thread::sleep(Duration::from_millis(50));
        }

        assert!(
            matches!(job.status(OWNER_1).ok(), Some(JobStatus::Done(ref status)) if status.success())
        );
    }

    #[tokio::test]
    async fn failing_command_status_is_done_failed() {
        let mut job = Job::new(Uuid::new_v4(), OWNER_1, "false").unwrap();

        while matches!(job.status(OWNER_1).ok(), Some(JobStatus::InProgress)) {
            thread::sleep(Duration::from_millis(50));
        }

        assert!(
            matches!(job.status(OWNER_1).ok(), Some(JobStatus::Done(ref status)) if !status.success())
        );
    }

    #[tokio::test]
    async fn command_stop_is_only_available_to_owner() {
        let mut job = Job::new(Uuid::new_v4(), OWNER_1, "true").unwrap();

        assert!(job.stop(OWNER_2).is_err());
        assert!(matches!(
            job.stop(OWNER_2).err(),
            Some(JobError::Unauthorized)
        ));
    }

    #[tokio::test]
    async fn long_running_command_can_be_stopped() {
        let mut job = Job::new(Uuid::new_v4(), OWNER_1, "sleep 100").unwrap();

        let mut i = 0;
        while matches!(job.status(OWNER_1).ok(), Some(JobStatus::InProgress)) {
            thread::sleep(Duration::from_millis(50));
            if i == 7 && job.stop(OWNER_1).is_err() {
                break;
            }
            i += 1;
        }

        assert!(i < 10);
        assert!(
            matches!(job.status(OWNER_1).ok(), Some(JobStatus::Done(ref status)) if !status.success())
        );
    }
}
