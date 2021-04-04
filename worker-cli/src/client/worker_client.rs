use super::{error::Error, request, response};
use reqwest::blocking::Client;
use std::{array::IntoIter, collections::HashMap};
use uuid::Uuid;

/// Type that defines the parameters for operating with `worker-api`
pub struct WorkerClient {
    base_url: String,
    endpoints: HashMap<String, String>,
}

impl WorkerClient {
    /// Create a new instance with the default base URL and endpoints.
    pub fn new() -> WorkerClient {
        WorkerClient {
            base_url: String::from("http://127.0.0.1:8000"),
            endpoints: IntoIter::new([
                ("login".to_string(), "/auth/login".to_string()),
                ("jobs".to_string(), "/v1/jobs".to_string()),
            ])
            .collect(),
        }
    }

    fn endpoint(&self, name: &str) -> Option<String> {
        let path = self.endpoints.get(name)?;
        Some(format!("{}{}", self.base_url, path))
    }

    /// Login for worker-api and return token.
    ///
    /// * `user` - User name.
    /// * `password` - User password.
    pub fn login(&self, user: String, password: String) -> Result<String, Error> {
        let endpoint = self.endpoint("login").ok_or(Error::InternalError)?;
        let client = Client::new();
        let login_request = request::Login {
            name: user,
            password,
        };
        let response = client.post(&endpoint).json(&login_request).send()?;

        if response.status().is_success() {
            let token_data = response.json::<response::Login>()?;
            Ok(token_data.token)
        } else {
            Err(Error::ApiError(response.status()))
        }
    }

    /// Start a job in worker-api.
    ///
    /// * `token` - authenticated JWT that is obtained from the login command.
    /// * `id` - valid UUID that will be used for the created job.
    /// * `command_line` - the command that will be executed in the job.
    pub fn start(&self, token: &str, id: Uuid, command_line: &str) -> Result<(), Error> {
        let endpoint = self.endpoint("jobs").ok_or(Error::InternalError)?;
        let client = Client::new();
        let new_job_request = request::NewJob {
            id,
            command_line: command_line.to_string(),
        };
        let response = client
            .post(&endpoint)
            .bearer_auth(token)
            .json(&new_job_request)
            .send()?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(Error::ApiError(response.status()))
        }
    }

    /// Status of a job in worker-api.
    ///
    /// * `token` - authenticated JWT that is obtained from the login command.
    /// * `id` - valid UUID of the job to be queried.
    pub fn status(&self, token: &str, id: Uuid) -> Result<String, Error> {
        let endpoint = self.endpoint("jobs").ok_or(Error::InternalError)?;
        let endpoint_with_id = format!("{}/{}", endpoint, id);
        let client = Client::new();
        let response = client.get(&endpoint_with_id).bearer_auth(token).send()?;

        if response.status().is_success() {
            let status_data = response.json::<response::Status>()?;
            let status = format!(
                "{} ({})",
                status_data.status,
                status_data
                    .exit_status
                    .map_or_else(|| "_".to_string(), |s| s.to_string())
            );
            Ok(status)
        } else {
            Err(Error::ApiError(response.status()))
        }
    }

    /// Output of a job in worker-api.
    ///
    /// * `token` - authenticated JWT that is obtained from the login command.
    /// * `id` - valid UUID of the job to be queried.
    pub fn output(&self, token: &str, id: Uuid) -> Result<String, Error> {
        let endpoint = self.endpoint("jobs").ok_or(Error::InternalError)?;
        let endpoint_with_id = format!("{}/{}/output", endpoint, id);
        let client = Client::new();
        let response = client.get(&endpoint_with_id).bearer_auth(token).send()?;

        if response.status().is_success() {
            let output_data = response.text()?;
            Ok(output_data)
        } else {
            Err(Error::ApiError(response.status()))
        }
    }

    /// Stop a job in worker-api.
    ///
    /// * `token` - authenticated JWT that is obtained from the login command.
    /// * `id` - valid UUID of the job to be stopped.
    pub fn stop(&self, token: &str, id: Uuid) -> Result<(), Error> {
        let endpoint = self.endpoint("jobs").ok_or(Error::InternalError)?;
        let endpoint_with_id = format!("{}/{}", endpoint, id);
        let client = Client::new();
        let response = client.delete(&endpoint_with_id).bearer_auth(token).send()?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(Error::ApiError(response.status()))
        }
    }
}
