use rocket::{get, http::Status};

#[get("/")]
pub async fn health() -> Status {
    // This will verify connections to other services or status of the resources
    Status::Ok
}
