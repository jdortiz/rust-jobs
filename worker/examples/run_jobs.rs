use std::{thread, time::Duration};
use uuid::Uuid;
use worker::{Job, JobError, JobStatus};

#[tokio::main]
async fn main() -> Result<(), JobError> {
    const OWNER_1: &str = "owner 1";
    const COMMAND_1: &str = "sleep 2";
    const COMMAND_2: &str = "sleep 10";

    let mut job1 = Job::new(Uuid::new_v4(), OWNER_1, COMMAND_1)?;
    println!("Spawned job 1 ('{}') with id={}", COMMAND_1, job1.get_id());
    let mut job2 = Job::new(Uuid::new_v4(), OWNER_1, COMMAND_2)?;
    println!("Spawned job 2 ('{}') with id={}", COMMAND_2, job2.get_id());

    // Wait for job 1 to finish
    while matches!(job1.status(OWNER_1).ok(), Some(JobStatus::InProgress)) {
        println!("Not done yet.");
        // This is a blocking sleep
        thread::sleep(Duration::from_millis(500));
    }
    // Now that job 1 is done, let's stop job 2
    job2.stop(OWNER_1)?;

    println!("Finished job 1. Status: {}", job1.status(OWNER_1)?);
    println!("Finished job 2. Status: {}", job2.status(OWNER_1)?);
    while matches!(job2.status(OWNER_1).ok(), Some(JobStatus::InProgress)) {
        println!("2 not done yet.");
        // This is a blocking sleep
        thread::sleep(Duration::from_millis(500));
    }
    println!("Finished job 2. Status: {}", job2.status(OWNER_1)?);

    Ok(())
}
