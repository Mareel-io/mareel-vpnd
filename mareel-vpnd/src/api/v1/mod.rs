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

use std::collections::HashMap;
use std::sync::{Mutex, RwLock, Arc};

use ::prometheus::{Encoder, TextEncoder};
use dashmap::{DashMap, DashSet};
use rocket::fairing::AdHoc;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{serde, Shutdown, State};
use rocket_client_addr::ClientAddr;

use crate::api::tokenauth::ApiKey;
use wgctrl::platform_specific::common::PlatformRoute;
use wgctrl::platform_specific::PlatformSpecificFactory;

use self::types::{DnsMonStore, IpStore, RouteManagerStore};

use super::common::{ApiResponse, ApiResponseType, PrometheusStore};

use types::{DaemonControlMessage, InterfaceStore};

mod interface;
mod peer;
mod route;
mod types;

#[post("/shutdown", format = "json", data = "<magic>")]
async fn shutdown_daemon(
    _apikey: ApiKey,
    shutdown: Shutdown,
    iface_store: &State<InterfaceStore>,
    magic: Json<DaemonControlMessage>,
) -> ApiResponseType<String> {
    match magic.magic {
        0xfee1dead => {
            // Shutdown
            let ifaces = &iface_store.iface_states;
            let keys: Vec<String> = { ifaces.iter().map(|x| x.key().clone()).collect() };

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
    pub(crate) version: String,
    pub(crate) magic: String,
}

#[get("/heartbeat")]
async fn heartbeat() -> (Status, Json<HeartbeatMessage>) {
    (
        Status::Ok,
        Json(HeartbeatMessage {
            version: env!("CARGO_PKG_VERSION").to_string(),
            magic: "0x4e6f6374696c756361".into(),
        }),
    )
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(crate = "rocket::serde")]
struct MyipMessage {
    pub(crate) ipv4: Option<String>,
    pub(crate) ipv6: Option<String>,
}

#[get("/myip")]
async fn myip(client_addr: &ClientAddr) -> (Status, Json<MyipMessage>) {
    (
        Status::Ok,
        Json::from(MyipMessage {
            ipv4: client_addr.get_ipv4_string(),
            ipv6: Some(client_addr.get_ipv6_string()),
        }),
    )
}

#[get("/prometheus")]
async fn prometheus(
    _apikey: ApiKey,
    iface_store: &State<InterfaceStore>,
    prom_store: &State<PrometheusStore>,
) -> (Status, String) {
    let ifaces = &iface_store.iface_states;

    // Update

    for iface in ifaces.iter() {
        let ifacestat = iface.lock().unwrap();
        let trafficstat = match ifacestat.interface.get_trafficstats() {
            Ok(x) => x,
            Err(e) => return (Status::InternalServerError, e.to_string()),
        };

        // Move to HashMap
        let mut hm: HashMap<String, (u64, u64)> = HashMap::new();
        for stat in trafficstat.iter() {
            hm.insert(stat.pubkey.clone(), (stat.tx_bytes, stat.rx_bytes));
        }

        for (peer, tx_cnt, rx_cnt) in ifacestat.peer_cfgs.values() {
            if let Some((tx_bytes, rx_bytes)) = hm.get(&peer.pubkey) {
                if (*tx_bytes as f64 - tx_cnt.get()) > 0.0 {
                    tx_cnt.inc_by(*tx_bytes as f64 - tx_cnt.get());
                } else {
                    tx_cnt.reset();
                    tx_cnt.inc_by(*tx_bytes as f64);
                }

                if (*rx_bytes as f64 - rx_cnt.get()) > 0.0 {
                    rx_cnt.inc_by(*rx_bytes as f64 - rx_cnt.get());
                } else {
                    rx_cnt.reset();
                    rx_cnt.inc_by(*rx_bytes as f64);
                }
            }
        }
    }

    let reg = prom_store.registry.lock().unwrap();
    let mut buffer = Vec::<u8>::new();
    let encoder = TextEncoder::new();
    let metric_families = reg.gather();

    drop(reg);

    encoder.encode(&metric_families, &mut buffer).unwrap();
    (Status::Ok, String::from_utf8(buffer).unwrap())
}

// TODO: FIXME: Hacky way to integrate talpid anyway
lazy_static! {
    static ref TALPID_TOKIO_RT: rocket::tokio::runtime::Runtime =
        rocket::tokio::runtime::Runtime::new().unwrap();
}

pub(crate) fn stage() -> AdHoc {
    AdHoc::on_ignite("API v1", |rocket| async {
        let mut route_manager = Box::new(PlatformSpecificFactory::get_route(0x7370616b).unwrap());
        match route_manager.init() {
            Ok(_) => {}
            Err(_) => {
                panic!("Failed to initialize RouteManager!")
            }
        }
        rocket
            .mount(
                "/api/v1",
                routes![
                    shutdown_daemon,
                    heartbeat,
                    myip,
                    interface::create_iface,
                    interface::get_ifaces,
                    interface::get_iface,
                    //interface::update_iface,
                    interface::delete_iface,
                    interface::get_status,
                    interface::put_status,
                    interface::put_ips,
                    interface::post_routes,
                    interface::get_routes,
                    interface::delete_routes,
                    interface::get_trafficstat,
                    interface::put_dns,
                    interface::delete_dns,
                    peer::create_peer,
                    peer::get_peers,
                    peer::get_peer,
                    //peer::update_peer,
                    peer::delete_peer,
                    route::create_bypass,
                    route::get_bypass,
                    route::delete_bypass,
                    prometheus,
                ],
            )
            .manage(InterfaceStore {
                iface_states: DashMap::new(),
            })
            .manage(RouteManagerStore {
                route_manager: Mutex::new(route_manager),
                route_store: DashMap::new(),
            })
            .manage(DnsMonStore {
                dnsmon: Arc::new(Mutex::new(
                    PlatformSpecificFactory::get_dnsmon(TALPID_TOKIO_RT.handle().clone()).unwrap(),
                )),
            })
            .manage(IpStore {
                v4: DashSet::new(),
                v4_last_count: RwLock::new(0),
                v6: DashSet::new(),
                v6_last_count: RwLock::new(0),
            })
    })
}
