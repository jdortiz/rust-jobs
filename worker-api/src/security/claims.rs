use super::control;
use rocket::{
    http::Status,
    request::{FromRequest, Outcome, Request},
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Claims {
    pub sub: String,
    pub user_name: String,
    pub iat: usize,
    pub exp: usize,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Claims {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if let Some(auth_header) = request.headers().get_one("Authorization") {
            let auth_str = auth_header.to_string();
            if let Some(token) = auth_str.strip_prefix("Bearer").map(|s| s.trim()) {
                if let Some(claims) = control::decode_token(token) {
                    return Outcome::Success(claims);
                }
            }
        }
        Outcome::Failure((Status::Unauthorized, ()))
    }
}
