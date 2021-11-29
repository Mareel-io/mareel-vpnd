use rocket::{Rocket, Build};
use rocket::fairing::AdHoc;

mod interface;
mod peer;

pub(crate) fn stage() -> AdHoc {
    AdHoc::on_ignite("API v1", |rocket| async {
        rocket.mount("/api/v1", routes![
            interface::create_iface,
            interface::get_ifaces,
            interface::get_iface,
            interface::update_iface,
            interface::delete_iface,
            peer::create_peer,
            peer::get_peers,
            peer::get_peer,
            peer::update_peer,
            peer::delete_peer,
        ])
    })
}