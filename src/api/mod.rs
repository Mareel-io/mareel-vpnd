use rocket::fairing::AdHoc;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{Build, Rocket};

pub(crate) mod common;
mod v1;

pub(crate) fn stage() -> AdHoc {
    AdHoc::on_ignite("API", |rocket| async { rocket.attach(v1::stage()) })
}
