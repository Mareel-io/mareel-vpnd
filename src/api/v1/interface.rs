use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::api::common::{ApiError, ApiResponse};
use crate::vpnctrl::platform_specific::common::{InterfaceStatus, PlatformInterface, WgIfCfg};
use crate::vpnctrl::platform_specific::PlatformSpecificFactory;
use rocket::serde::json::Json;
use rocket::State;
use rocket::{http::Status, serde};

use super::tokenauth::ApiKey;
use super::{IfaceState, InterfaceConfig, InterfaceStore};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(crate = "rocket::serde")]
pub(crate) struct InterfaceStatusResp {
    pub(crate) status: String,
}

#[post("/interface", format = "json", data = "<ifcfg>")]
pub(crate) async fn create_iface(
    _apikey: ApiKey,
    iface_store: &State<InterfaceStore>,
    ifcfg: Json<InterfaceConfig>,
) -> (Status, Result<Json<ApiResponse<String>>, Json<ApiError>>) {
    let private_key = match ifcfg.private_key.clone() {
        Some(pk) => pk,
        _ => {
            return (
                Status::BadRequest,
                Err(Json(ApiError {
                    code: -1,
                    msg: "Cannot create interface without its name nor private key".to_string(),
                })),
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
            Err(Json(ApiError {
                code: -1,
                msg: "Cannot create interface with same name".to_string(),
            })),
        );
    }

    // Create interface
    let iface = match PlatformSpecificFactory::get_interface(&ifcfg.name) {
        Ok(mut x) => {
            match x.set_config(WgIfCfg {
                listen_port: None,
                privkey: private_key,
            }) {
                Ok(_) => Box::new(x),
                Err(_e) => {
                    return (
                        Status::BadRequest,
                        Err(Json(ApiError {
                            code: -1,
                            msg: "foo".to_string(),
                        })),
                    )
                }
            }
        }
        Err(e) => {
            return (
                Status::InternalServerError,
                Err(Json(ApiError {
                    code: -1,
                    msg: e.to_string(),
                })),
            )
        }
    };

    iface_store.iface_states.lock().unwrap().insert(
        ifcfg.name.clone(),
        Arc::new(Mutex::new(IfaceState {
            interface: iface,
            iface_cfg: ifcfg.into_inner(),
            peer_cfgs: HashMap::new(),
        })),
    );

    let ret: ApiResponse<String> = ApiResponse {
        status: Some("ok".to_string()),
        data: None,
    };
    (Status::Ok, Ok(Json(ret)))
}

#[get("/interface")]
pub(crate) async fn get_ifaces(
    _apikey: ApiKey,
    iface_store: &State<InterfaceStore>,
) -> Option<Json<Vec<InterfaceConfig>>> {
    Some(Json(
        iface_store
            .iface_states
            .lock()
            .unwrap()
            .values()
            .map(|x| x.lock().unwrap().iface_cfg.clone())
            .collect(),
    ))
}

#[get("/interface/<id>")]
pub(crate) async fn get_iface(
    _apikey: ApiKey,
    iface_store: &State<InterfaceStore>,
    id: String,
) -> (Status, Option<Json<InterfaceConfig>>) {
    match iface_store.iface_states.lock().unwrap().get(&id) {
        Some(x) => (Status::Ok, Some(Json(x.lock().unwrap().iface_cfg.clone()))),
        None => (Status::NotFound, None),
    }
}

#[put("/interface/<id>", format = "json", data = "<ifcfg>")]
pub(crate) async fn update_iface(
    _apikey: ApiKey,
    id: String,
    ifcfg: Json<InterfaceConfig>,
) -> (Status, Option<Json<String>>) {
    (Status::NotImplemented, None)
}

#[delete("/interface/<id>")]
pub(crate) async fn delete_iface(
    _apikey: ApiKey,
    iface_store: &State<InterfaceStore>,
    id: String,
) -> (Status, Option<Json<String>>) {
    let mut ifaces = iface_store.iface_states.lock().unwrap();
    match ifaces.get(&id) {
        Some(x) => {
            let mut iface = x.lock().unwrap();
            iface.interface.down();
            iface.interface.delete();
            drop(iface);
            ifaces.remove(&id);
            (Status::Ok, Some(Json("ok".to_string())))
        }
        None => (Status::NotFound, None),
    }
}

// Interface startup/shutdown
#[get("/interface/<id>/status")]
pub(crate) async fn get_status(
    _apikey: ApiKey,
    iface_store: &State<InterfaceStore>,
    id: String,
) -> (Status, Option<Json<InterfaceStatusResp>>) {
    match iface_store.iface_states.lock().unwrap().get(&id) {
        Some(x) => (
            Status::Ok,
            Some(Json(InterfaceStatusResp {
                status: x.lock().unwrap().interface.get_status().to_string(),
            })),
        ),
        None => (Status::NotFound, None),
    }
}

#[put("/interface/<id>/status", format = "json", data = "<status>")]
pub(crate) async fn put_status(
    _apikey: ApiKey,
    iface_store: &State<InterfaceStore>,
    id: String,
    status: Json<InterfaceStatusResp>,
) -> (Status, Option<Json<InterfaceStatusResp>>) {
    let next_stat = match status.status.as_str() {
        "start" => InterfaceStatus::Running,
        "stop" => InterfaceStatus::Stopped,
        _ => return (Status::BadRequest, None),
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
                Some(Json(InterfaceStatusResp {
                    status: intf.get_status().to_string(),
                })),
            )
        }
        None => (Status::NotFound, None),
    }
}
