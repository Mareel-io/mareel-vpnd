use std::sync::{Arc, Mutex};

use prometheus::Registry;
use rocket::fairing::AdHoc;

use self::common::PrometheusStore;

pub(crate) mod common;
pub(crate) mod tokenauth;
mod v1;

pub(crate) struct AuthKeyProvider {
    auth_key: String,
}

pub(crate) fn stage(key: &str, registry: Arc<Mutex<Registry>>) -> AdHoc {
    let k = key.to_owned();
    AdHoc::on_ignite("API", |rocket| async {
        rocket
            .attach(v1::stage())
            .manage(AuthKeyProvider { auth_key: k })
            .manage(PrometheusStore { registry })
    })
}
