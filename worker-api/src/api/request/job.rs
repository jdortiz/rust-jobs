use rocket_contrib::uuid::Uuid;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Job {
    pub id: Uuid,
    pub command_line: String,
}
