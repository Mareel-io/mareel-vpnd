use rocket::{http::Status, serde::json::Json, State};

use crate::{
    api::{
        common::{ApiResponse, ApiResponseType},
        v1::InterfaceStore,
    },
    vpnctrl::platform_specific::common::WgPeerCfg,
};

use super::types::PeerConfig;
use crate::api::tokenauth::ApiKey;

#[post("/interface/<if_id>/peer", format = "json", data = "<peercfg>")]
pub(crate) async fn create_peer(
    _apikey: ApiKey,
    iface_store: &State<InterfaceStore>,
    if_id: String,
    peercfg: Json<PeerConfig>,
) -> ApiResponseType<String> {
    let iface_states = iface_store.iface_states.lock().unwrap();
    let mut iface_state = match iface_states.get(&if_id) {
        Some(x) => x,
        None => return (Status::NotFound, ApiResponse::err(-1, "Not found")),
    }
    .lock()
    .unwrap();

    if iface_state.peer_cfgs.get(&peercfg.pubkey).is_some() {
        return (Status::Conflict, ApiResponse::err(-1, "Conflict"));
    };

    // Do some magic
    match iface_state.interface.add_peer(WgPeerCfg {
        pubkey: peercfg.pubkey.clone(),
        psk: None,
        endpoint: peercfg.endpoint.clone(),
        allowed_ips: peercfg.allowed_ips.clone(),
        keep_alive: peercfg.keepalive,
    }) {
        Ok(_) => {}
        Err(e) => {
            return (
                Status::InternalServerError,
                ApiResponse::err(-1, &e.to_string()),
            )
        }
    }

    iface_state
        .peer_cfgs
        .insert(peercfg.pubkey.clone(), peercfg.into_inner());

    (Status::Ok, ApiResponse::ok("Ok".to_string()))
}

#[get("/interface/<if_id>/peer")]
pub(crate) async fn get_peers(
    _apikey: ApiKey,
    iface_store: &State<InterfaceStore>,
    if_id: String,
) -> ApiResponseType<Vec<PeerConfig>> {
    let iface_states = iface_store.iface_states.lock().unwrap();
    let iface_state = match iface_states.get(&if_id) {
        Some(x) => x,
        None => return (Status::NotFound, ApiResponse::err(-1, "Not found")),
    }
    .lock()
    .unwrap();

    let peers: Vec<PeerConfig> = iface_state.peer_cfgs.values().cloned().collect();

    (Status::Ok, ApiResponse::ok(peers))
}

#[get("/interface/<if_id>/peer/<pubk>")]
pub(crate) async fn get_peer(
    _apikey: ApiKey,
    iface_store: &State<InterfaceStore>,
    if_id: String,
    pubk: String,
) -> ApiResponseType<PeerConfig> {
    let iface_states = iface_store.iface_states.lock().unwrap();
    let iface_state = match iface_states.get(&if_id) {
        Some(x) => x,
        None => return (Status::NotFound, ApiResponse::err(-1, "Not found")),
    }
    .lock()
    .unwrap();

    match iface_state.peer_cfgs.get(&pubk) {
        Some(x) => (Status::Ok, ApiResponse::ok(x.clone())),
        None => return (Status::NotFound, ApiResponse::err(-1, "Not found")),
    }
}

//#[put("/interface/<if_id>/peer/<pubk>", format = "json", data = "<peercfg>")]
//pub(crate) async fn update_peer(
//    _apikey: ApiKey,
//    iface_store: &State<InterfaceStore>,
//    if_id: String,
//    pubk: String,
//    peercfg: Json<PeerConfig>,
//) -> Option<Json<String>> {
//    None
//}

#[delete("/interface/<if_id>/peer/<pubk>")]
pub(crate) async fn delete_peer(
    _apikey: ApiKey,
    iface_store: &State<InterfaceStore>,
    if_id: String,
    pubk: String,
) -> ApiResponseType<String> {
    let iface_states = iface_store.iface_states.lock().unwrap();
    let mut iface_state = match iface_states.get(&if_id) {
        Some(x) => x,
        None => return (Status::NotFound, ApiResponse::err(-1, "Not found")),
    }
    .lock()
    .unwrap();

    if iface_state.peer_cfgs.get(&pubk).is_none() {
        return (Status::NotFound, ApiResponse::err(-1, "Not found"));
    };

    iface_state.peer_cfgs.remove(&pubk);
    match iface_state.interface.remove_peer(&pubk) {
        Ok(_) => (),
        Err(e) => {
            return (
                Status::InternalServerError,
                ApiResponse::err(-1, &e.to_string()),
            )
        }
    };

    (Status::Ok, ApiResponse::ok("Peer removed".to_string()))
}
