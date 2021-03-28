use std::{fmt::Display, process::ExitStatus};

#[derive(Clone)]
/// Custom status type for the jobs.
pub enum JobStatus {
    /// The job has been launched and it is still being executed.
    InProgress,
    /// The child process of the job has had a problem and cannot be queried.
    Failed,
    /// The job has finished executing and the exit status can be checked.
    Done(ExitStatus),
}

impl Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            JobStatus::InProgress => write!(f, "IN_PROGRESS"),
            JobStatus::Failed => write!(f, "FAILED"),
            JobStatus::Done(ref status) => write!(f, "DONE({})", status),
        }
    }
}
