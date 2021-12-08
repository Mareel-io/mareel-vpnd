use rocket::fairing::AdHoc;

pub(crate) mod common;
mod v1;

pub(crate) fn stage() -> AdHoc {
    AdHoc::on_ignite("API", |rocket| async { rocket.attach(v1::stage()) })
}
