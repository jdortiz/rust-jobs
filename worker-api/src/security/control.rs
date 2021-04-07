use super::{claims::Claims, user::User};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::env;

pub fn login(user: User) -> Option<String> {
    let uid;
    // FIXME: The authentication is hardcoded for demonstration
    // purposes. DON'T use in a production system.
    match user.name.as_ref() {
        "jorge" => {
            uid = String::from("f14f83b7-c626-4255-9da4-cec1ac22b4a1");
        }
        "joel" => {
            uid = String::from("c0e38e26-8364-4bac-aed0-f0463945557b");
        }
        _ => {
            return None;
        }
    }

    let now = Utc::now();
    let expiration = now + Duration::days(1);

    let claims = Claims {
        sub: uid,
        user_name: user.name,
        iat: now.timestamp() as usize,
        exp: expiration.timestamp() as usize,
    };
    let header = Header::default();
    let secret = get_secret();
    encode(&header, &claims, &EncodingKey::from_secret(secret.as_ref())).ok()
}

pub fn decode_token(token: &str) -> Option<Claims> {
    let secret = get_secret();
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .map(|token_data| token_data.claims)
    .ok()
}

fn get_secret() -> String {
    const JWT_SECRET: &str = "secret";
    let secret = env::var("WORKER_API_SECRET").unwrap_or_else(|_| JWT_SECRET.to_string());
    if secret == JWT_SECRET {
        eprintln!("USING DEFAULT SECRET: THIS INSECURE BEHAVIOR IS ONLY MEANT FOR TESTING");
    }

    secret
}
