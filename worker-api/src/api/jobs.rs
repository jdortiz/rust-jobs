use super::{request, response};
use crate::{security::Claims, JobData};
use rocket::{delete, get, http::Status, post, response::NamedFile, State};
use rocket_contrib::{json::Json, uuid::Uuid};
use worker::{Job, JobError, JobStatus};

#[post("/", format = "application/json", data = "<new_job>")]
pub async fn create(
    claims: Claims,
    new_job: Json<request::Job>,
    jobs: State<'_, JobData>,
) -> Status {
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
pub async fn get(
    claims: Claims,
    job_id: Uuid,
    jobs: State<'_, JobData>,
) -> Result<Json<response::JobStatus>, Status> {
    eprintln!("claim subject: {}", claims.sub);
    eprintln!("Job to query: {:?}", job_id);
    let mut jobs_map = jobs.write().unwrap();
    if let Some(job) = jobs_map.get_mut(&job_id.into_inner()) {
        match job.status(&claims.sub) {
            Ok(status) => match status {
                JobStatus::Failed | JobStatus::InProgress => Ok(Json(response::JobStatus {
                    status: status.to_string(),
                    exit_status: None,
                })),
                JobStatus::Done(exit_value) => Ok(Json(response::JobStatus {
                    status: status.to_string(),
                    exit_status: exit_value.code(),
                })),
            },
            Err(err) if matches!(err, JobError::Unauthorized) => Err(Status::Forbidden),
            _ => Err(Status::InternalServerError),
        }
    } else {
        Err(Status::NotFound)
    }
}

#[get("/<job_id>/output")]
pub async fn get_output(
    claims: Claims,
    job_id: Uuid,
    jobs: State<'_, JobData>,
) -> Result<NamedFile, Status> {
    eprintln!("claim subject: {}", claims.sub);
    eprintln!("Job to query: {:?}", job_id);
    let filename: Result<String, Status> = {
        let mut jobs_map = jobs.write().unwrap();
        if let Some(job) = jobs_map.get_mut(&job_id.into_inner()) {
            job.output(&claims.sub).map_err(|err| match err {
                JobError::Unauthorized => Status::Forbidden,
                _ => Status::InternalServerError,
            })
        } else {
            Err(Status::NotFound)
        }
    };
    match filename {
        Ok(filename) => NamedFile::open(&filename)
            .await
            .map_err(|_| Status::InternalServerError),
        _ => Err(Status::NotFound),
    }
}

#[delete("/<job_id>")]
pub async fn delete(claims: Claims, job_id: Uuid, jobs: State<'_, JobData>) -> Status {
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
