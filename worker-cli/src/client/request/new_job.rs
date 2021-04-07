use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub struct NewJob {
    pub id: Uuid,
    pub command_line: String,
}
