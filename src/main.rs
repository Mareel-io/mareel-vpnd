use std::net::IpAddr;
use std::str::FromStr;

use rocket::config::Config;

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate lazy_static;

mod api;
mod vpnctrl;

#[launch]
fn rocket() -> _ {
    let cfg = Config {
        address: IpAddr::from_str("0.0.0.0").unwrap(),
        port: 8080,
        ..Default::default()
    };

    rocket::custom(cfg).attach(api::stage())
}
