use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use rocket::fairing::AdHoc;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{serde, Build, Rocket, State};

use crate::vpnctrl::platform_specific::common::PlatformInterface;

use super::common::ApiError;

mod interface;
mod peer;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(crate = "rocket::serde")]
pub(crate) struct InterfaceConfig {
    pub(crate) name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) private_key: Option<String>,
    #[serde(skip_deserializing)]
    pub(crate) public_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) listen_port: Option<u16>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(crate = "rocket::serde")]
pub(crate) struct PeerConfig {
    pub(crate) pubk: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) psk: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) endpoint: Option<String>,
    pub(crate) allowed_ips: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) keepalive: Option<i64>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(crate = "rocket::serde")]
struct DaemonControlMessage {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub(crate) magic: Option<u32>,
}

pub(crate) struct InterfaceStore {
    iface_config_map: Mutex<HashMap<String, Arc<Mutex<InterfaceConfig>>>>,
    ifaces: Mutex<HashMap<String, Arc<Mutex<Box<dyn PlatformInterface + Send>>>>>,
}

#[post("/shutdown", format = "json", data = "<magic>")]
fn shutdown_daemon(
    iface_store: &State<InterfaceStore>,
    magic: Json<DaemonControlMessage>,
) -> (Status, Result<Json<String>, Json<ApiError>>) {
    match magic.magic {
        Some(0xfee1dead) => {}
        _ => {
            return (
                Status::BadRequest,
                Err(Json(ApiError {
                    code: -1,
                    msg: "Bad magic number".to_string(),
                })),
            );
        }
    }

    todo!("Implement shutdown logic");

    (Status::Ok, Ok(Json("All is well".to_string())))
}

pub(crate) fn stage() -> AdHoc {
    AdHoc::on_ignite("API v1", |rocket| async {
        rocket
            .mount(
                "/api/v1",
                routes![
                    shutdown_daemon,
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
