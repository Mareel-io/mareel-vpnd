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
use std::convert::TryInto;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};

use crate::api::common::{ApiResponse, ApiResponseType, PrometheusStore};
use crate::api::v1::types::DnsMonStore;
use rocket::serde::json::Json;
use rocket::State;
use rocket::{http::Status, serde};
use wgctrl::platform_specific::common::{
    InterfaceStatus, PeerTrafficStat, PlatformInterface, PlatformRoute, WgIfCfg,
};
use wgctrl::platform_specific::PlatformSpecificFactory;

// Raw crypto primitives
use curve25519_dalek::constants::ED25519_BASEPOINT_TABLE;
use curve25519_dalek::scalar::Scalar;

use super::types::{
    IfaceState, InterfaceConfig, InterfaceStore, IpConfigurationMessage, RouteConfigurationMessage,
    RouteManagerStore,
};
use crate::api::tokenauth::ApiKey;

// Some helper functions
fn extract_pubkey(private_key: &str) -> Result<String, String> {
    let pk_bytes: [u8; 32] = match base64::decode(private_key) {
        Ok(x) => match (x.as_slice().try_into()) as Result<[u8; 32], _> {
            Ok(mut x) => {
                // Apply key clamping
                // TODO: Is it really safe? Research more curve25519 cryptography and find out.
                x[0] &= 248;
                x[31] &= 127;
                x[31] |= 64;
                x
            }
            Err(_) => return Err("Bad private key: wrong size".to_string()),
        },
        Err(_) => return Err("Bad private key: not in b64 format!".to_string()),
    };

    let point = (&ED25519_BASEPOINT_TABLE * &Scalar::from_bits(pk_bytes)).to_montgomery();
    Ok(base64::encode(point.to_bytes()))
}

#[test]
fn test_extract_pubkey_normal() {
    let privk = "ADD7fFbGmA0TqivcbwW7RACosgn2ZqK5uDSijvUul2c=";
    let pubk = "LCBsla9u/BT2i9yYKqCi6yHh2nKvvdgyMPVYCkLh/3Y=";

    let our_pubk = extract_pubkey(privk).unwrap();

    assert_eq!(our_pubk, pubk);
}

#[test]
fn test_extract_pubkey_clamp() {
    let privk = "gGHF8XEpNKEnzIjoQNs6CRy5bVBTR8ZMcWbFckkWiv8=";
    let pubk = "zxUOG5Sb+wZY70iCiK5R4oeTuf1IC/e1whg8GkHl5hI=";

    let our_pubk = extract_pubkey(privk).unwrap();

    assert_eq!(our_pubk, pubk);
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(crate = "rocket::serde")]
pub(crate) struct InterfaceStatusResp {
    pub(crate) status: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(crate = "rocket::serde")]
pub(crate) struct DnsConfigureReq {
    pub dns: Vec<String>,
}

#[post("/interface", format = "json", data = "<ifcfg>")]
pub(crate) async fn create_iface(
    _apikey: ApiKey,
    rms: &State<RouteManagerStore>,
    iface_store: &State<InterfaceStore>,
    ifcfg: Json<InterfaceConfig>,
) -> ApiResponseType<String> {
    let private_key = match ifcfg.private_key.clone() {
        Some(pk) => pk,
        _ => {
            return (
                Status::BadRequest,
                ApiResponse::err(
                    -1,
                    "Cannot create interface without its name nor private key",
                ),
            );
        }
    };

    let public_key = match extract_pubkey(&private_key) {
        Ok(x) => x,
        Err(msg) => return (Status::UnprocessableEntity, ApiResponse::err(-1, &msg)),
    };

    if iface_store.iface_states.get(&ifcfg.name).is_some() {
        return (
            Status::Conflict,
            ApiResponse::err(-1, "Cannot create interface with same name"),
        );
    }

    let iface_states = &iface_store.iface_states;

    if iface_states.is_empty() {
        // No keys found. back up the route!
        let mut rm = rms.route_manager.lock().unwrap();
        match rm.backup_default_route() {
            Ok(_) => {}
            Err(x) => {
                return (
                    Status::InternalServerError,
                    ApiResponse::err(-1, &x.to_string()),
                );
            }
        }
    }

    // Create interface
    let iface = match PlatformSpecificFactory::get_interface(&ifcfg.name) {
        Ok(mut x) => {
            match x.set_config(WgIfCfg {
                listen_port: ifcfg.listen_port,
                privkey: private_key,
                fwmark: 0x7370616b,
            }) {
                Ok(_) => Box::new(x),
                Err(e) => {
                    return (Status::BadRequest, ApiResponse::err(-1, &e.to_string()));
                }
            }
        }
        Err(e) => {
            return (
                Status::InternalServerError,
                ApiResponse::err(-1, &e.to_string()),
            )
        }
    };

    let mut iface_cfg: InterfaceConfig = ifcfg.into_inner();

    // For security reason, do not hold private_key in return object
    iface_cfg.private_key = None;
    iface_cfg.public_key = Some(public_key);

    iface_states.insert(
        iface_cfg.name.clone(),
        Arc::new(Mutex::new(IfaceState {
            interface: iface,
            iface_cfg,
            peer_cfgs: HashMap::new(),
        })),
    );

    (Status::Ok, ApiResponse::ok("ok".to_string()))
}

#[get("/interface")]
pub(crate) async fn get_ifaces(
    _apikey: ApiKey,
    iface_store: &State<InterfaceStore>,
) -> ApiResponseType<Vec<InterfaceConfig>> {
    (
        Status::Ok,
        ApiResponse::ok(
            iface_store
                .iface_states
                .iter()
                .map(|x| x.lock().unwrap().iface_cfg.clone())
                .collect(),
        ),
    )
}

#[get("/interface/<id>")]
pub(crate) async fn get_iface(
    _apikey: ApiKey,
    iface_store: &State<InterfaceStore>,
    id: String,
) -> ApiResponseType<InterfaceConfig> {
    match iface_store.iface_states.get(&id) {
        Some(x) => (
            Status::Ok,
            ApiResponse::ok(x.lock().unwrap().iface_cfg.clone()),
        ),
        None => (Status::NotFound, ApiResponse::err(-1, "Not found")),
    }
}

//#[put("/interface/<id>", format = "json", data = "<ifcfg>")]
//pub(crate) async fn update_iface(
//    _apikey: ApiKey,
//    id: String,
//    ifcfg: Json<InterfaceConfig>,
//) -> (Status, Option<Json<String>>) {
//    (Status::NotImplemented, None)
//}

#[delete("/interface/<id>")]
pub(crate) async fn delete_iface(
    _apikey: ApiKey,
    rms: &State<RouteManagerStore>,
    iface_store: &State<InterfaceStore>,
    prom_store: &State<PrometheusStore>,
    id: String,
) -> ApiResponseType<String> {
    let ifaces = &iface_store.iface_states;
    let mut rm = rms.route_manager.lock().unwrap();
    let rs = &rms.route_store;
    let reg = prom_store.registry.lock().unwrap();
    match rm.restore_default_route() {
        Ok(_) => {}
        Err(_x) => {
            return (
                Status::InternalServerError,
                ApiResponse::err(-1, "Uh-oh. :("),
            );
        }
    }

    // Remove all route owned by the interface
    rs.remove(&id);

    match ifaces.get(&id) {
        Some(x) => {
            let mut iface = x.lock().unwrap();
            iface.interface.down();
            for (_, tx_cnt, rx_cnt) in iface.peer_cfgs.values() {
                reg.unregister(Box::new(tx_cnt.clone())).unwrap();
                reg.unregister(Box::new(rx_cnt.clone())).unwrap();
            }

            // Wait for iface drop explictly
            drop(iface);
            drop(x);
            ifaces.remove(&id);
            (Status::Ok, ApiResponse::ok("Ok".to_string()))
        }
        None => (Status::NotFound, ApiResponse::err(-1, "Not found")),
    }
}

// Interface startup/shutdown
#[get("/interface/<id>/status")]
pub(crate) async fn get_status(
    _apikey: ApiKey,
    iface_store: &State<InterfaceStore>,
    id: String,
) -> ApiResponseType<InterfaceStatusResp> {
    match iface_store.iface_states.get(&id) {
        Some(x) => (
            Status::Ok,
            ApiResponse::ok(InterfaceStatusResp {
                status: x.lock().unwrap().interface.get_status().to_string(),
            }),
        ),
        None => (Status::NotFound, ApiResponse::err(-1, "Not found")),
    }
}

#[put("/interface/<id>/status", format = "json", data = "<status>")]
pub(crate) async fn put_status(
    _apikey: ApiKey,
    iface_store: &State<InterfaceStore>,
    id: String,
    status: Json<InterfaceStatusResp>,
) -> ApiResponseType<InterfaceStatusResp> {
    let next_stat = match status.status.as_str() {
        "start" => InterfaceStatus::Running,
        "stop" => InterfaceStatus::Stopped,
        _ => return (Status::BadRequest, ApiResponse::err(-1, "Bad Request")),
    };

    match iface_store.iface_states.get(&id) {
        Some(x) => {
            let intf = &mut x.lock().unwrap().interface;
            let cur_stat = intf.get_status();

            match (cur_stat, next_stat) {
                (InterfaceStatus::Stopped, InterfaceStatus::Running) => {
                    intf.up();
                }
                (InterfaceStatus::Running, InterfaceStatus::Stopped) => {
                    intf.down();
                }
                (_, _) => {}
            };
            (
                Status::Ok,
                ApiResponse::ok(InterfaceStatusResp {
                    status: intf.get_status().to_string(),
                }),
            )
        }
        None => (Status::NotFound, ApiResponse::err(-1, "Not found")),
    }
}

#[put("/interface/<id>/ips", format = "json", data = "<ips>")]
pub(crate) async fn put_ips(
    _apikey: ApiKey,
    iface_store: &State<InterfaceStore>,
    id: String,
    ips: Json<IpConfigurationMessage>,
) -> ApiResponseType<String> {
    match iface_store.iface_states.get(&id) {
        Some(x) => {
            let intf = &mut x.lock().unwrap().interface;
            match intf.set_ip(&ips.ipaddr) {
                Ok(_) => (Status::Ok, ApiResponse::ok("Ok".to_string())),
                Err(e) => (
                    Status::InternalServerError,
                    ApiResponse::err(-1, &e.to_string()),
                ),
            }
        }
        None => (Status::NotFound, ApiResponse::err(-1, "Not found")),
    }
}

#[post("/interface/<id>/routes", format = "json", data = "<route>")]
pub(crate) async fn post_routes(
    _apikey: ApiKey,
    iface_store: &State<InterfaceStore>,
    rms: &State<RouteManagerStore>,
    id: String,
    route: Json<RouteConfigurationMessage>,
) -> ApiResponseType<String> {
    match iface_store.iface_states.get(&id) {
        Some(_) => {
            let mut rm = rms.route_manager.lock().unwrap();
            let rs = &rms.route_store;
            let mut routemap = match rs.get_mut(&id) {
                Some(x) => x,
                None => {
                    rs.insert(id.clone(), HashMap::new());
                    rs.get_mut(&id).unwrap()
                }
            };

            // Search map before adding CIDR
            match routemap.get(&route.cidr) {
                Some(_) => {
                    return (
                        Status::Conflict,
                        ApiResponse::err(-1, "Route conflict. cannot add it"),
                    )
                }
                None => {
                    routemap.insert(route.cidr.clone(), true);
                }
            }

            // Try to put route CIDR to route_store
            if route.cidr == "0.0.0.0/0" {
                match rm.remove_default_route() {
                    Ok(_) => (),
                    Err(e) => {
                        return (
                            Status::InternalServerError,
                            ApiResponse::err(-1, &e.to_string()),
                        )
                    }
                }
            }

            match rm.add_route(&id, &route.cidr) {
                Ok(_) => (Status::Ok, ApiResponse::ok("Ok".to_string())),
                Err(e) => (
                    Status::InternalServerError,
                    ApiResponse::err(-1, &e.to_string()),
                ),
            }
        }
        None => (Status::NotFound, ApiResponse::err(-1, "Not found")),
    }
}

#[get("/interface/<id>/routes")]
pub(crate) async fn get_routes(
    _apikey: ApiKey,
    iface_store: &State<InterfaceStore>,
    rms: &State<RouteManagerStore>,
    id: String,
) -> ApiResponseType<Vec<String>> {
    if iface_store.iface_states.get(&id).is_none() {
        return (Status::NotFound, ApiResponse::err(-1, "Not found"));
    }

    let rs = &rms.route_store;
    let routemap = match rs.get(&id) {
        Some(x) => x,
        None => {
            rs.insert(id.clone(), HashMap::new());
            rs.get(&id).unwrap()
        }
    };

    let keys: Vec<String> = routemap.keys().cloned().collect();

    (Status::Ok, ApiResponse::ok(keys))
}

#[delete("/interface/<id>/routes/<cidr>")]
pub(crate) async fn delete_routes(
    _apikey: ApiKey,
    iface_store: &State<InterfaceStore>,
    rms: &State<RouteManagerStore>,
    id: String,
    cidr: String,
) -> ApiResponseType<String> {
    if iface_store.iface_states.get(&id).is_none() {
        return (Status::NotFound, ApiResponse::err(-1, "IFace not found"));
    }

    let mut rm = rms.route_manager.lock().unwrap();
    let rs = &rms.route_store;
    let mut routemap = match rs.get_mut(&id) {
        Some(x) => x,
        None => {
            return (Status::NotFound, ApiResponse::err(-1, "CIDR not found"));
        }
    };

    match routemap.remove(&cidr) {
        Some(_) => match rm.remove_route(&id, &cidr) {
            Ok(_) => (Status::Ok, ApiResponse::ok("Ok".to_string())),
            Err(e) => (
                Status::InternalServerError,
                ApiResponse::err(-1, &e.to_string()),
            ),
        },
        None => (Status::NotFound, ApiResponse::err(-1, "CIDR not found")),
    }
}

#[get("/interface/<id>/traffic")]
pub(crate) async fn get_trafficstat(
    _apikey: ApiKey,
    iface_store: &State<InterfaceStore>,
    id: String,
) -> ApiResponseType<Vec<PeerTrafficStat>> {
    match iface_store.iface_states.get(&id) {
        Some(x) => {
            let intf = &mut x.lock().unwrap().interface;
            match intf.get_trafficstats() {
                Ok(x) => (Status::Ok, ApiResponse::ok(x)),
                Err(e) => (
                    Status::InternalServerError,
                    ApiResponse::err(-1, &e.to_string()),
                ),
            }
        }
        None => (Status::NotFound, ApiResponse::err(-1, "Not found")),
    }
}

#[put("/interface/<id>/dns", format = "json", data = "<dns>")]
pub(crate) async fn put_dns(
    _apikey: ApiKey,
    iface_store: &State<InterfaceStore>,
    dns_store: &State<DnsMonStore>,
    id: String,
    dns: Json<DnsConfigureReq>,
) -> ApiResponseType<String> {
    let platformid = match iface_store.iface_states.get(&id) {
        Some(x) => match x.lock().unwrap().interface.get_platformid() {
            Ok(id) => id,
            Err(e) => {
                return (Status::InternalServerError, ApiResponse::err(-1, ":("));
            }
        },
        None => {
            return (Status::NotFound, ApiResponse::err(-1, "Not found"));
        }
    };

    let dns: Vec<IpAddr> = match dns.dns.iter().map(|e| e.parse()).collect() {
        Ok(x) => x,
        Err(e) => {
            return (
                Status::UnprocessableEntity,
                ApiResponse::err(-1, &e.to_string()),
            );
        }
    };

    let dnsmon_lock = dns_store.dnsmon.clone();
    match rocket::tokio::task::spawn_blocking(move || {
        let mut dnsmon = dnsmon_lock.lock().unwrap();
        dnsmon.set(&platformid, &dns)
    })
    .await
    {
        Ok(_) => (Status::Ok, ApiResponse::ok("ok".to_string())),
        Err(e) => (
            Status::InternalServerError,
            ApiResponse::err(-1, &e.to_string()),
        ),
    }
}

#[delete("/interface/<id>/dns")]
pub(crate) async fn delete_dns(
    _apikey: ApiKey,
    iface_store: &State<InterfaceStore>,
    dns_store: &State<DnsMonStore>,
    id: String,
) -> ApiResponseType<String> {
    let platformid = match iface_store.iface_states.get(&id) {
        Some(x) => match x.lock().unwrap().interface.get_platformid() {
            Ok(id) => id,
            Err(e) => {
                return (Status::InternalServerError, ApiResponse::err(-1, ":("));
            }
        },
        None => {
            return (Status::NotFound, ApiResponse::err(-1, "Not found"));
        }
    };

    let dnsmon_lock = dns_store.dnsmon.clone();
    match rocket::tokio::task::spawn_blocking(move || {
        let mut dnsmon = dnsmon_lock.lock().unwrap();
        dnsmon.reset()
    })
    .await
    {
        Ok(_) => (Status::Ok, ApiResponse::ok("ok".to_string())),
        Err(e) => (
            Status::InternalServerError,
            ApiResponse::err(-1, &e.to_string()),
        ),
    }
}
