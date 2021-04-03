use crate::security::{self, Token, User};
use rocket::{http::Status, post};
use rocket_contrib::json::Json;

/// HTTP handler for user login.
///
/// * user: `User` - User data (name and password).
#[post("/login", format = "application/json", data = "<user>")]
pub async fn login(user: Json<User>) -> Result<Json<Token>, Status> {
    //! This is a hardwired process at the moment, it should check
    //! hashes in a user database to authenticate the user.
    let user = user.into_inner();
    eprintln!("Login user: {:?}", user);
    // https://github.com/rust-lang/rust-clippy/issues/7024
    #[allow(clippy::suspicious_else_formatting)]
    if let Some(token) = security::login(user) {
        Ok(Json(Token { token }))
    } else {
        Err(Status::Unauthorized)
    }
}
