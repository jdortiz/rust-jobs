use uuid::Uuid;
use worker::{Job, JobError};

fn main() -> Result<(), JobError> {
    let job = Job::new(Uuid::new_v4(), "ls -l")?;

    println!("Spawned job with id={}", job.get_id());

    Ok(())
}
