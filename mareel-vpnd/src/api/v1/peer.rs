/*
 * SPDX-FileCopyrightText: 2022 Empo Inc.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful, but
 * WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
 * General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

use prometheus::{Counter, Opts};
use regex::Regex;
use rocket::{http::Status, serde::json::Json, State};

use crate::api::{
    common::{ApiResponse, ApiResponseType, PrometheusStore},
    v1::{
        types::{IpStore, RouteManagerStore},
        InterfaceStore,
    },
};
use wgctrl::platform_specific::common::{PlatformRoute, WgPeerCfg};

use super::types::PeerConfig;
use crate::api::tokenauth::ApiKey;

#[post("/interface/<if_id>/peer", format = "json", data = "<peercfg>")]
pub(crate) async fn create_peer(
    _apikey: ApiKey,
    rms: &State<RouteManagerStore>,
    iface_store: &State<InterfaceStore>,
    ip_store: &State<IpStore>,
    prom_store: &State<PrometheusStore>,
    if_id: String,
    mut peercfg: Json<PeerConfig>,
) -> ApiResponseType<PeerConfig> {
    // Check allowed_ips is CIDR or not
    let cidr_re = Regex::new(r"([0-9a-fA-F:.]+/[0-9]+)").unwrap();
    for allowed_ip in peercfg.allowed_ips.iter() {
        if !cidr_re.is_match(allowed_ip) {
            return (
                Status::UnprocessableEntity,
                ApiResponse::err(-1, "allowed_ips contains non-CIDR formatted entry"),
            );
        }
    }

    let iface_states = &iface_store.iface_states;
    let iface_state_lock = match iface_states.get(&if_id) {
        Some(x) => x,
        None => return (Status::NotFound, ApiResponse::err(-1, "Not found")),
    };

    let mut iface_state = iface_state_lock.lock().unwrap();

    if iface_state.peer_cfgs.get(&peercfg.pubkey).is_some() {
        return (Status::Conflict, ApiResponse::err(-1, "Conflict"));
    };

    if Some(true) == peercfg.autoalloc {
        let v4store = &ip_store.v4;
        let v6store = &ip_store.v6;
        let mut v4_last_count = ip_store.v4_last_count.write().unwrap();

        let mut v4_suffix: u32 = 0;
        for _i in 1..0x1000000 {
            *v4_last_count = match *v4_last_count {
                0 => 2,
                0xFFFFFF.. => 2,
                _ => *v4_last_count + 1,
            };

            // Check existance
            if v4store.get(&*v4_last_count).is_none() {
                v4store.insert(*v4_last_count);
                v4_suffix = *v4_last_count;
                break;
            }
        }

        if v4_suffix == 0 {
            return (
                Status::NotAcceptable,
                ApiResponse::err(-1, "Resource not available"),
            );
        }

        let mut v6_last_count = ip_store.v6_last_count.write().unwrap();
        let mut v6_suffix: u64 = 0;
        for _i in 1u64..0x100000000u64 {
            *v6_last_count = match *v6_last_count {
                0 => 2,
                0xFFFFFFFF.. => 2,
                _ => *v6_last_count + 1,
            };

            // Check existance
            if v6store.get(&*v6_last_count).is_none() {
                v6store.insert(*v6_last_count);
                v6_suffix = *v6_last_count;
                break;
            }
        }

        if v6_suffix == 0 {
            return (
                Status::NotAcceptable,
                ApiResponse::err(-1, "Resource not available"),
            );
        }

        peercfg.allowed_ips = Vec::new();
        peercfg.allowed_ips.push(format!(
            "10.{}.{}.{}/32",
            (v4_suffix & 0xFF0000) >> 16,
            (v4_suffix & 0xFF00) >> 8,
            v4_suffix & 0xFF
        ));
        peercfg.allowed_ips.push(format!(
            "fd92:6943:1c6e:96bc::{:x}:{:x}/128",
            (v6_suffix & 0xFFFF0000) >> 16,
            v6_suffix & 0xFFFF,
        ));

        peercfg.autoalloc_v4 = Some(v4_suffix);
        peercfg.autoalloc_v6 = Some(v6_suffix);
    }

    if let Some(endpt) = &peercfg.endpoint {
        let mut rm = rms.route_manager.lock().unwrap();
        let re = Regex::new(r":.*").unwrap();
        let ip = re.replace_all(endpt, "");
        match rm.add_route_bypass(&(*ip).to_string()) {
            Ok(_) => {}
            Err(_x) => {
                return (
                    Status::InternalServerError,
                    ApiResponse::err(-1, "Failed to bypass peer endpt"),
                );
            }
        }
    }

    let peer_tx_opts = Opts::new("peer_tx", "Peer TX bytes")
        .const_label("interface", if_id.clone())
        .const_label("pubk", peercfg.pubkey.clone());
    let peer_rx_opts = Opts::new("peer_rx", "Peer RX bytes")
        .const_label("interface", if_id.clone())
        .const_label("pubk", peercfg.pubkey.clone());

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
    let tx_counter = Counter::with_opts(peer_tx_opts).unwrap();
    let rx_counter = Counter::with_opts(peer_rx_opts).unwrap();

    let reg = prom_store.registry.lock().unwrap();
    reg.register(Box::new(tx_counter.clone())).unwrap();
    reg.register(Box::new(rx_counter.clone())).unwrap();
    drop(reg);

    iface_state.peer_cfgs.insert(
        peercfg.pubkey.clone(),
        (peercfg.clone(), tx_counter, rx_counter),
    );

    (Status::Ok, ApiResponse::ok(peercfg.into_inner()))
}

#[get("/interface/<if_id>/peer")]
pub(crate) async fn get_peers(
    _apikey: ApiKey,
    iface_store: &State<InterfaceStore>,
    if_id: String,
) -> ApiResponseType<Vec<PeerConfig>> {
    let iface_states = &iface_store.iface_states;
    let iface_state_lock = match iface_states.get(&if_id) {
        Some(x) => x,
        None => return (Status::NotFound, ApiResponse::err(-1, "Not found")),
    };

    let iface_state = iface_state_lock.lock().unwrap();

    let peers: Vec<PeerConfig> = iface_state
        .peer_cfgs
        .values()
        .map(|x| x.0.clone())
        .collect();

    (Status::Ok, ApiResponse::ok(peers))
}

#[get("/interface/<if_id>/peer/<pubk>")]
pub(crate) async fn get_peer(
    _apikey: ApiKey,
    iface_store: &State<InterfaceStore>,
    if_id: String,
    pubk: String,
) -> ApiResponseType<PeerConfig> {
    let iface_states = &iface_store.iface_states;
    let iface_state_lock = match iface_states.get(&if_id) {
        Some(x) => x,
        None => return (Status::NotFound, ApiResponse::err(-1, "Not found")),
    };

    let iface_state = iface_state_lock.lock().unwrap();

    match iface_state.peer_cfgs.get(&pubk) {
        Some(x) => (Status::Ok, ApiResponse::ok(x.0.clone())),
        None => (Status::NotFound, ApiResponse::err(-1, "Not found")),
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
    prom_store: &State<PrometheusStore>,
    if_id: String,
    pubk: String,
) -> ApiResponseType<String> {
    let iface_states = &iface_store.iface_states;
    let iface_state_lock = match iface_states.get(&if_id) {
        Some(x) => x,
        None => return (Status::NotFound, ApiResponse::err(-1, "Not found")),
    };
    let mut iface_state = iface_state_lock.lock().unwrap();

    let (peercfg, tx_counter, rx_counter) = match iface_state.peer_cfgs.get(&pubk) {
        Some(x) => x.clone(),
        None => {
            return (Status::NotFound, ApiResponse::err(-1, "Not found"));
        }
    };

    if let Some(_endpt) = &peercfg.endpoint {
        //let mut rm = rms.route_manager.lock().unwrap();
        //match rm.delete_route_bypass(&endpt) {
        //    Ok(_) => {}
        //    Err(_x) => {
        //        //return (
        //        //    Status::InternalServerError,
        //        //    ApiResponse::err(-1, "Failed to bypass peer endpt"),
        //        //);
        //    }
        //}
    }

    let reg = prom_store.registry.lock().unwrap();
    reg.unregister(Box::new(tx_counter)).ok();
    reg.unregister(Box::new(rx_counter)).ok();
    drop(reg);

    iface_state.peer_cfgs.remove(&pubk);
    match iface_state.interface.remove_peer(&pubk) {
        Ok(_) => {
            if let Some(x) = peercfg.autoalloc_v4 {
                let v4store = &ip_store.v4;
                v4store.remove(&x);
            }
            if let Some(x) = peercfg.autoalloc_v6 {
                let v4store = &ip_store.v6;
                v4store.remove(&x);
            }
        }
        Err(e) => {
            return (
                Status::InternalServerError,
                ApiResponse::err(-1, &e.to_string()),
            )
        }
    };

    (Status::Ok, ApiResponse::ok("Peer removed".to_string()))
}
