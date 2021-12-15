use rocket::{http::Status, serde::json::Json, State};

use crate::{
    api::{
        common::{ApiResponse, ApiResponseType},
        v1::{types::IpStore, InterfaceStore},
    },
    vpnctrl::platform_specific::common::WgPeerCfg,
};

use super::types::PeerConfig;
use crate::api::tokenauth::ApiKey;

#[post("/interface/<if_id>/peer", format = "json", data = "<peercfg>")]
pub(crate) async fn create_peer(
    _apikey: ApiKey,
    iface_store: &State<InterfaceStore>,
    ip_store: &State<IpStore>,
    if_id: String,
    mut peercfg: Json<PeerConfig>,
) -> ApiResponseType<PeerConfig> {
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

    if Some(true) == peercfg.autoalloc {
        let mut v4store = ip_store.v4.lock().unwrap();
        let mut v4_last_count = ip_store.v4_last_count.lock().unwrap();

        let mut ip_suffix: u32 = 0;
        for _i in 1..0x1000000 {
            *v4_last_count = match *v4_last_count {
                0 => 2,
                0xFFFFFF.. => 2,
                _ => *v4_last_count + 1,
            };

            // Check existance
            if v4store.get(&*v4_last_count).is_none() {
                v4store.insert(*v4_last_count, true);
                ip_suffix = *v4_last_count;
                break;
            }
        }

        if ip_suffix == 0 {
            return (
                Status::NotAcceptable,
                ApiResponse::err(-1, "Resource not available"),
            );
        }

        peercfg.allowed_ips = Vec::new();
        peercfg.allowed_ips.push(format!(
            "10.{}.{}.{}/32",
            ip_suffix & 0xFF0000,
            ip_suffix & 0xFF00,
            ip_suffix & 0xFF
        ));

        peercfg.autoalloc_v4 = Some(ip_suffix);
    }

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
        .insert(peercfg.pubkey.clone(), peercfg.clone());

    (Status::Ok, ApiResponse::ok(peercfg.into_inner()))
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
    ip_store: &State<IpStore>,
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

    let peercfg = match iface_state.peer_cfgs.get(&pubk) {
        Some(x) => x.clone(),
        None => {
            return (Status::NotFound, ApiResponse::err(-1, "Not found"));
        }
    };

    iface_state.peer_cfgs.remove(&pubk);
    match iface_state.interface.remove_peer(&pubk) {
        Ok(_) => {
            if let Some(x) = peercfg.autoalloc_v4 {
                let mut v4store = ip_store.v4.lock().unwrap();
                v4store.remove(&x);
            }
            if let Some(x) = peercfg.autoalloc_v6 {
                let mut v4store = ip_store.v6.lock().unwrap();
                v4store.remove(&x);
            }
        },
        Err(e) => {
            return (
                Status::InternalServerError,
                ApiResponse::err(-1, &e.to_string()),
            )
        }
    };

    (Status::Ok, ApiResponse::ok("Peer removed".to_string()))
}
