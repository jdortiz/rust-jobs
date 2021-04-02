use serde::Serialize;

#[derive(Serialize)]
pub struct Login {
    pub name: String,
    pub password: String,
}
