use std::collections::HashMap;
use std::sync::Mutex;

use rocket::fairing::AdHoc;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{serde, Shutdown, State};

use super::common::{ApiResponse, ApiResponseType};

use types::{DaemonControlMessage, InterfaceStore};

mod interface;
mod peer;
mod types;

#[post("/shutdown", format = "json", data = "<magic>")]
async fn shutdown_daemon(
    shutdown: Shutdown,
    iface_store: &State<InterfaceStore>,
    magic: Json<DaemonControlMessage>,
) -> ApiResponseType<String> {
    match magic.magic {
        0xfee1dead => {
            // Shutdown
            let mut ifaces = iface_store.iface_states.lock().unwrap();
            let keys: Vec<String> = { ifaces.keys().cloned().collect() };

            for k in keys {
                if let Some(x) = ifaces.get(&k) {
                    x.lock().unwrap().interface.down();
                    ifaces.remove(&k);
                }
            }

            shutdown.notify();
            (Status::Ok, ApiResponse::ok("All is well".to_string()))
        }
        _ => (Status::BadRequest, ApiResponse::err(-1, "Bad Magic Number")),
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(crate = "rocket::serde")]
struct HeartbeatMessage {
    pub(crate) magic: String,
}

#[get("/heartbeat")]
async fn heartbeat() -> (Status, Json<HeartbeatMessage>) {
    (
        Status::Ok,
        Json(HeartbeatMessage {
            magic: "0x4e6f6374696c756361".into(),
        }),
    )
}

pub(crate) fn stage() -> AdHoc {
    AdHoc::on_ignite("API v1", |rocket| async {
        rocket
            .mount(
                "/api/v1",
                routes![
                    shutdown_daemon,
                    heartbeat,
                    interface::create_iface,
                    interface::get_ifaces,
                    interface::get_iface,
                    //interface::update_iface,
                    interface::delete_iface,
                    interface::get_status,
                    interface::put_status,
                    interface::put_ips,
                    interface::post_routes,
                    interface::get_trafficstat,
                    peer::create_peer,
                    peer::get_peers,
                    peer::get_peer,
                    //peer::update_peer,
                    peer::delete_peer,
                ],
            )
            .manage(InterfaceStore {
                iface_states: Mutex::new(HashMap::new()),
            })
    })
}
