mod api;
mod security;

use api::{auth, jobs};
use rocket::{get, launch, routes};
use std::{collections::HashMap, sync::RwLock};
use uuid::Uuid;
use worker;

#[get("/health")]
fn health() -> &'static str {
    // This will verify connections to other services or status of the resources
    "Ok"
}

type JobData = RwLock<HashMap<Uuid, worker::Job>>;

#[launch]
fn rocket() -> rocket::Rocket {
    let data: JobData = RwLock::new(HashMap::new());
    rocket::ignite()
        .manage(data)
        .mount("/", routes![health])
        .mount("/auth", routes![auth::login])
        .mount("/v1/jobs", routes![jobs::create, jobs::get, jobs::delete])
}
