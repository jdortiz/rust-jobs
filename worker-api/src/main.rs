mod api;
mod security;

use api::{auth, health, jobs};
use rocket::{launch, routes};
use std::{collections::HashMap, sync::RwLock};
use uuid::Uuid;

type JobData = RwLock<HashMap<Uuid, worker::Job>>;

#[launch]
fn rocket() -> rocket::Rocket {
    let data: JobData = RwLock::new(HashMap::new());
    rocket::ignite()
        .manage(data)
        .mount("/health", routes![health::health])
        .mount("/auth", routes![auth::login])
        .mount(
            "/v1/jobs",
            routes![jobs::create, jobs::get, jobs::get_output, jobs::delete],
        )
}
