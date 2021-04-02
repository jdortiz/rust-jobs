use serde::Deserialize;

#[derive(Deserialize)]
pub struct Login {
    pub token: String,
}
