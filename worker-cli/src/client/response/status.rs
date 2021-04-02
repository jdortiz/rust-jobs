use serde::Deserialize;

#[derive(Deserialize)]
pub struct Status {
    pub status: String,
    pub exit_status: Option<i32>,
}
