/*!
This is a library that allows to run commands, query their state and
their output and stop them.

It provides an abstraction over a job.
*/
mod job;
mod job_error;

pub use job::Job;
pub use job_error::JobError;
