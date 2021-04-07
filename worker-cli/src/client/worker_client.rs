use super::{error::Error, request, response};
use reqwest::{blocking::Client, Certificate};
use std::{array::IntoIter, collections::HashMap, time::Duration};
use std::{fs::File, io::Read};
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
            base_url: String::from("https://localhost:8000"),
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

    fn customized_client() -> Result<Client, Error> {
        let mut buf = Vec::new();
        File::open("private/rsacert.pem")?.read_to_end(&mut buf)?;
        let cert = Certificate::from_pem(&buf)?;
        let client = Client::builder()
            .add_root_certificate(cert)
            .https_only(true)
            // .danger_accept_invalid_certs(true) // TLS: Required for macOS
            .timeout(Some(Duration::from_secs(5)))
            .build()?;

        Ok(client)
    }

    /// Login for worker-api and return token.
    ///
    /// * `user` - User name.
    /// * `password` - User password.
    pub fn login(&self, user: &str, password: &str) -> Result<String, Error> {
        let endpoint = self.endpoint("login").ok_or(Error::InternalError)?;
        let client = Self::customized_client()?;
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

    /// Start a job in worker-api.
    ///
    /// * `token` - authenticated JWT that is obtained from the login command.
    /// * `id` - valid UUID that will be used for the created job.
    /// * `command_line` - the command that will be executed in the job.
    pub fn start(&self, token: &str, id: Uuid, command_line: &str) -> Result<(), Error> {
        let endpoint = self.endpoint("jobs").ok_or(Error::InternalError)?;
        let client = Self::customized_client()?;
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
        let client = Self::customized_client()?;
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
        let client = Self::customized_client()?;
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
        let client = Self::customized_client()?;
        let response = client.delete(&endpoint_with_id).bearer_auth(token).send()?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(Error::ApiError(response.status()))
        }
    }
}
