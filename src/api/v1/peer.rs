use rocket::{http::Status, serde::json::Json, State};

use crate::{
    api::{common::ApiResponse, v1::InterfaceStore},
    vpnctrl::platform_specific::common::WgPeerCfg,
};

use super::PeerConfig;

#[post("/interface/<if_id>/peer", format = "json", data = "<peercfg>")]
pub(crate) async fn create_peer(
    iface_store: &State<InterfaceStore>,
    if_id: String,
    peercfg: Json<PeerConfig>,
) -> (Status, Option<Json<ApiResponse<String>>>) {
    let iface_states = iface_store.iface_states.lock().unwrap();
    let mut iface_state = match iface_states.get(&if_id) {
        Some(x) => x,
        None => return (Status::NotFound, None),
    }
    .lock()
    .unwrap();

    if iface_state.peer_cfgs.get(&peercfg.pubk).is_some() {
        return (Status::Conflict, None);
    };

    // Do some magic
    match iface_state.interface.add_peer(WgPeerCfg {
        pubkey: peercfg.pubk.clone(),
        psk: None,
        endpoint: peercfg.endpoint.clone(),
        allowed_ips: peercfg.allowed_ips.clone(),
        keep_alive: peercfg.keepalive,
    }) {
        Ok(_) => {}
        Err(_) => return (Status::InternalServerError, None),
    }

    iface_state
        .peer_cfgs
        .insert(peercfg.pubk.clone(), peercfg.into_inner());

    let ret: ApiResponse<String> = ApiResponse {
        status: Some("ok".to_string()),
        data: None,
    };
    (Status::Ok, Some(Json(ret)))
}

#[get("/interface/<if_id>/peer")]
pub(crate) async fn get_peers(
    iface_store: &State<InterfaceStore>,
    if_id: String,
) -> (Status, Option<Json<Vec<PeerConfig>>>) {
    let iface_states = iface_store.iface_states.lock().unwrap();
    let iface_state = match iface_states.get(&if_id) {
        Some(x) => x,
        None => return (Status::NotFound, None),
    }
    .lock()
    .unwrap();

    let peers: Vec<PeerConfig> = iface_state.peer_cfgs.values().cloned().collect();

    (Status::Ok, Some(Json(peers)))
}

#[get("/interface/<if_id>/peer/<pubk>")]
pub(crate) async fn get_peer(
    iface_store: &State<InterfaceStore>,
    if_id: String,
    pubk: String,
) -> Option<Json<PeerConfig>> {
    None
}

#[put("/interface/<if_id>/peer/<pubk>", format = "json", data = "<peercfg>")]
pub(crate) async fn update_peer(
    iface_store: &State<InterfaceStore>,
    if_id: String,
    pubk: String,
    peercfg: Json<PeerConfig>,
) -> Option<Json<String>> {
    None
}

#[delete("/interface/<if_id>/peer/<pubk>")]
pub(crate) async fn delete_peer(
    iface_store: &State<InterfaceStore>,
    if_id: String,
    pubk: String,
) -> Option<String> {
    None
}
