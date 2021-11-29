use rocket::{Rocket, Build};
use rocket::fairing::AdHoc;
use rocket::serde::{Serialize, Deserialize, json::Json};

pub(crate) mod common;
mod v1;

pub(crate) fn stage() -> AdHoc {
    AdHoc::on_ignite("API", |rocket| async {
        rocket.attach(v1::stage())
    })
}