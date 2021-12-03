use std::sync::{Arc, Mutex};

use crate::api::common::{ApiError, ApiResponse};
use crate::vpnctrl::platform_specific::common::{InterfaceStatus, PlatformInterface, WgIfCfg};
use crate::vpnctrl::platform_specific::PlatformSpecificFactory;
use rocket::serde::json::Json;
use rocket::State;
use rocket::{http::Status, serde};

use super::{InterfaceConfig, InterfaceStore};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(crate = "rocket::serde")]
pub(crate) struct InterfaceStatusResp {
    pub(crate) status: String,
}

#[post("/interface", format = "json", data = "<ifcfg>")]
pub(crate) async fn create_iface(
    iface_store: &State<InterfaceStore>,
    ifcfg: Json<InterfaceConfig>,
) -> (Status, Result<Json<ApiResponse<String>>, Json<ApiError>>) {
    let (name, private_key) = match (ifcfg.name.clone(), ifcfg.private_key.clone()) {
        (Some(a), Some(b)) => (a, b),
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

    // Create interface
    let iface = match PlatformSpecificFactory::get_interface(&name) {
        Ok(x) => Box::new(x),
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

    iface_store
        .ifaces
        .lock()
        .unwrap()
        .insert(name.clone(), Arc::new(Mutex::new(iface)));

    iface_store
        .iface_config_map
        .lock()
        .unwrap()
        .insert(name, Arc::new(Mutex::new(ifcfg.into_inner())));

    let ret: ApiResponse<String> = ApiResponse {
        status: Some("ok".to_string()),
        data: None,
    };
    (Status::Ok, Ok(Json(ret)))
}

#[get("/interface")]
pub(crate) async fn get_ifaces(
    iface_store: &State<InterfaceStore>,
) -> Option<Json<Vec<InterfaceConfig>>> {
    Some(Json(
        iface_store
            .iface_config_map
            .lock()
            .unwrap()
            .values()
            .map(|x| x.lock().unwrap().clone())
            .collect(),
    ))
}

#[get("/interface/<id>")]
pub(crate) async fn get_iface(
    iface_store: &State<InterfaceStore>,
    id: String,
) -> (Status, Option<Json<InterfaceConfig>>) {
    match iface_store.iface_config_map.lock().unwrap().get(&id) {
        Some(x) => (Status::Ok, Some(Json(x.lock().unwrap().clone()))),
        None => (Status::NotFound, None),
    }
}

#[put("/interface/<id>", format = "json", data = "<ifcfg>")]
pub(crate) async fn update_iface(
    id: String,
    ifcfg: Json<InterfaceConfig>,
) -> (Status, Option<Json<String>>) {
    (Status::NotImplemented, None)
}

#[delete("/interface/<id>")]
pub(crate) async fn delete_iface(
    iface_store: &State<InterfaceStore>,
    id: String,
) -> (Status, Option<Json<String>>) {
    let mut ifaces = iface_store.ifaces.lock().unwrap();
    match ifaces.get(&id) {
        Some(x) => {
            x.lock().unwrap().down();
            ifaces.remove(&id);
            iface_store.iface_config_map.lock().unwrap().remove(&id);
            (Status::Ok, Some(Json("ok".to_string())))
        }
        None => (Status::NotFound, None),
    }
}

// Interface startup/shutdown
#[get("/interface/<id>/status")]
pub(crate) async fn get_status(
    iface_store: &State<InterfaceStore>,
    id: String,
) -> (Status, Option<Json<InterfaceStatusResp>>) {
    match iface_store.ifaces.lock().unwrap().get(&id) {
        Some(x) => (
            Status::Ok,
            Some(Json(InterfaceStatusResp {
                status: x.lock().unwrap().get_status().to_string(),
            })),
        ),
        None => (Status::NotFound, None),
    }
}

#[put("/interface/<id>/status", format = "json", data = "<status>")]
pub(crate) async fn put_status(
    iface_store: &State<InterfaceStore>,
    id: String,
    status: Json<InterfaceStatusResp>,
) -> (Status, Option<Json<InterfaceStatusResp>>) {
    let next_stat = match status.status.as_str() {
        "start" => InterfaceStatus::RUNNING,
        "stop" => InterfaceStatus::STOPPED,
        _ => return (Status::BadRequest, None),
    };

    match iface_store.ifaces.lock().unwrap().get(&id) {
        Some(x) => {
            let intf = x.lock().unwrap();
            let cur_stat = intf.get_status();

            match (cur_stat, next_stat) {
                (InterfaceStatus::STOPPED, InterfaceStatus::RUNNING) => {
                    intf.up();
                }
                (InterfaceStatus::RUNNING, InterfaceStatus::STOPPED) => {
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
