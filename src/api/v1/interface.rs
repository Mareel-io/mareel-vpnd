use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::api::common::{ApiResponse, ApiResponseType, PrometheusStore};
use crate::vpnctrl::platform_specific::common::{
    InterfaceStatus, PeerTrafficStat, PlatformInterface, PlatformRoute, WgIfCfg,
};
use crate::vpnctrl::platform_specific::PlatformSpecificFactory;
use rocket::serde::json::Json;
use rocket::State;
use rocket::{http::Status, serde};

use super::types::{
    IfaceState, InterfaceConfig, InterfaceStore, IpConfigurationMessage, RouteConfigurationMessage,
    RouteManagerStore,
};
use crate::api::tokenauth::ApiKey;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(crate = "rocket::serde")]
pub(crate) struct InterfaceStatusResp {
    pub(crate) status: String,
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

    if iface_store
        .iface_states
        .get(&ifcfg.name)
        .is_some()
    {
        return (
            Status::Conflict,
            ApiResponse::err(-1, "Cannot create interface with same name"),
        );
    }

    let iface_states = &iface_store.iface_states;

    if iface_states.len() == 0 {
        // No keys found. back up the route!
        let mut rm = rms.route_manager.lock().unwrap();
        match rm.backup_default_route() {
            Ok(_) => {}
            Err(_x) => {
                return (
                    Status::InternalServerError,
                    ApiResponse::err(-1, "Uh-oh. :("),
                );
            }
        }
    }

    // Create interface
    let iface = match async {PlatformSpecificFactory::get_interface(&ifcfg.name)}.await {
        Ok(mut x) => {
            match x.set_config(WgIfCfg {
                listen_port: ifcfg.listen_port,
                privkey: private_key,
                fwmark: 0x7370616b,
            }) {
                Ok(_) => Box::new(x),
                Err(_e) => {
                    return (Status::BadRequest, ApiResponse::err(-1, "Uh-oh. :("));
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

    iface_states.insert(
        ifcfg.name.clone(),
        Arc::new(Mutex::new(IfaceState {
            interface: iface,
            iface_cfg: ifcfg.into_inner(),
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
        Some(x) => {
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
    if let None = iface_store.iface_states.get(&id) {
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

    let keys: Vec<String> = routemap.keys().map(|x| x.clone()).collect();

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
    if let None = iface_store.iface_states.get(&id) {
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
