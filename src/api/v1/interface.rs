use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::api::common::{ApiResponse, ApiResponseType};
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
        .lock()
        .unwrap()
        .get(&ifcfg.name)
        .is_some()
    {
        return (
            Status::Conflict,
            ApiResponse::err(-1, "Cannot create interface with same name"),
        );
    }

    let mut iface_states = iface_store.iface_states.lock().unwrap();

    if iface_states.keys().len() == 0 {
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
    let iface = match PlatformSpecificFactory::get_interface(&ifcfg.name) {
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
                .lock()
                .unwrap()
                .values()
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
    match iface_store.iface_states.lock().unwrap().get(&id) {
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
    id: String,
) -> ApiResponseType<String> {
    let mut ifaces = iface_store.iface_states.lock().unwrap();
    let mut rm = rms.route_manager.lock().unwrap();
    match rm.restore_default_route() {
        Ok(_) => {}
        Err(_x) => {
            return (
                Status::InternalServerError,
                ApiResponse::err(-1, "Uh-oh. :("),
            );
        }
    }
    match ifaces.get(&id) {
        Some(x) => {
            let mut iface = x.lock().unwrap();
            iface.interface.down();
            drop(iface);
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
    match iface_store.iface_states.lock().unwrap().get(&id) {
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

    match iface_store.iface_states.lock().unwrap().get(&id) {
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
    match iface_store.iface_states.lock().unwrap().get(&id) {
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
    match iface_store.iface_states.lock().unwrap().get(&id) {
        Some(x) => {
            let mut rm = rms.route_manager.lock().unwrap();
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

#[get("/interface/<id>/traffic")]
pub(crate) async fn get_trafficstat(
    _apikey: ApiKey,
    iface_store: &State<InterfaceStore>,
    id: String,
) -> ApiResponseType<Vec<PeerTrafficStat>> {
    match iface_store.iface_states.lock().unwrap().get(&id) {
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
