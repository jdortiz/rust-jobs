use super::{error::Error, request, response};
use reqwest::blocking::Client;
use std::{array::IntoIter, collections::HashMap};

pub struct WorkerClient {
    base_url: String,
    endpoints: HashMap<String, String>,
}

impl WorkerClient {
    pub fn new() -> WorkerClient {
        WorkerClient {
            base_url: String::from("http://127.0.0.1:8000"),
            endpoints: IntoIter::new([("login".to_string(), "/auth/login".to_string())]).collect(),
        }
    }

    fn endpoint(&self, name: &str) -> Option<String> {
        let path = self.endpoints.get(name)?;
        Some(format!("{}{}", self.base_url, path))
    }

    /// Login for worker-api and return token.
    pub fn login(&self, user: &str, password: &str) -> Result<String, Error> {
        let endpoint = self.endpoint("login").expect("Something went wrong");
        let client = Client::new();
        let login_request = request::Login {
            name: user.to_string(),
            password: password.to_string(),
        };
        let response = client.post(&endpoint).json(&login_request).send()?;

        if response.status().is_success() {
            let token_data = response.json::<response::Login>()?;
            Ok(token_data.token)
        } else {
            Err(Error::ApiError(response.status()))
        }
    }
}
