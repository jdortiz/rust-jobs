use serde::Serialize;

#[derive(Serialize)]
pub struct JobStatus {
    pub status: String,
    pub exit_status: Option<i32>,
}
