use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use rocket::fairing::AdHoc;
use rocket::{serde, Build, Rocket};

use crate::vpnctrl::platform_specific::common::PlatformInterface;

mod interface;
mod peer;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(crate = "rocket::serde")]
pub(crate) struct InterfaceConfig {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub(crate) name: Option<String>,
    pub(crate) private_key: Option<String>,
    pub(crate) public_key: Option<String>,
    pub(crate) listen_port: Option<u16>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(crate = "rocket::serde")]
pub(crate) struct PeerConfig {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub(crate) pubk: Option<String>,
    pub(crate) psk: Option<String>,
    pub(crate) endpoint: Option<String>,
    pub(crate) allowed_ips: Vec<String>,
    pub(crate) keepalive: Option<i64>,
}

pub(crate) struct InterfaceStore {
    iface_config_map: Mutex<HashMap<String, Arc<Mutex<InterfaceConfig>>>>,
    ifaces: Mutex<HashMap<String, Arc<Mutex<Box<dyn PlatformInterface + Send>>>>>,
}

pub(crate) fn stage() -> AdHoc {
    AdHoc::on_ignite("API v1", |rocket| async {
        rocket
            .mount(
                "/api/v1",
                routes![
                    interface::create_iface,
                    interface::get_ifaces,
                    interface::get_iface,
                    interface::update_iface,
                    interface::delete_iface,
                    interface::get_status,
                    interface::put_status,
                    peer::create_peer,
                    peer::get_peers,
                    peer::get_peer,
                    peer::update_peer,
                    peer::delete_peer,
                ],
            )
            .manage(InterfaceStore {
                ifaces: Mutex::new(HashMap::new()),
                iface_config_map: Mutex::new(HashMap::new()),
            })
    })
}
