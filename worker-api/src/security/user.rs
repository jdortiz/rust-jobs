use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct User {
    pub name: String,
    pub password: String,
}
