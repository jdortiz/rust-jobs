use crate::{security::Claims, JobData};
use rocket::{delete, get, http::Status, post, State};
use rocket_contrib::json::Json;
use rocket_contrib::uuid::{extern_uuid, Uuid};
use serde::Deserialize;
use worker::{Job, JobError};

#[derive(Deserialize, Debug)]
pub struct JobRequest {
    id: Uuid,
    command_line: String,
}

#[post("/", format = "application/json", data = "<new_job>")]
pub fn create(claims: Claims, new_job: Json<JobRequest>, jobs: State<JobData>) -> Status {
    eprintln!("claim subject: {}", claims.sub);
    eprintln!("New job: {:?}", new_job);
    let new_job = new_job.into_inner();
    let mut jobs_map = jobs.write().unwrap();
    if jobs_map.contains_key(&new_job.id.into_inner()) {
        Status::Conflict
    } else {
        match Job::new(new_job.id.into_inner(), &claims.sub, &new_job.command_line) {
            Ok(job) => {
                jobs_map.insert(new_job.id.into_inner(), job);
                Status::Created
            }
            Err(err) if matches!(err, JobError::CommandNotFound) => Status::BadRequest,
            Err(err) if matches!(err, JobError::InvalidCommand(_)) => Status::BadRequest,
            _ => Status::InternalServerError,
        }
    }
}

#[get("/<job_id>")]
pub fn get(claims: Claims, job_id: Uuid, jobs: State<JobData>) -> Status {
    eprintln!("claim subject: {}", claims.sub);
    eprintln!("Job to stop: {:?}", job_id);
    let mut jobs_map = jobs.write().unwrap();
    if let Some(job) = jobs_map.get_mut(&job_id.into_inner()) {
        match job.stop(&claims.sub) {
            Ok(()) => Status::Ok,
            Err(err) if matches!(err, JobError::Unauthorized) => Status::Forbidden,
            _ => Status::InternalServerError,
        }
    } else {
        Status::NotFound
    }
}

#[delete("/<job_id>")]
pub fn delete(claims: Claims, job_id: Uuid) -> Status {
    eprintln!("claim subject: {}", claims.sub);
    eprintln!("Job to stop: {:?}", job_id);
    if job_id.to_string().starts_with("62") {
        Status::Ok
    } else {
        Status::NotFound
    }
}
