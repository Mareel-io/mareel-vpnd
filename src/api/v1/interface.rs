use std::sync::{Arc, Mutex};

use crate::api::common::{ApiError, ApiResponse};
use crate::vpnctrl::platform_specific::common::{PlatformInterface, WgIfCfg};
use rocket::serde::json::Json;
use rocket::State;
use rocket::{http::Status, serde};

use super::{InterfaceConfig, InterfaceStore};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(crate = "rocket::serde")]
pub(crate) struct InterfaceStatus {
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
    //let ifaces = iface_store.ifaces.keys().into_iter().map(|x| {
    //});

    let ifaces = iface_store
        .ifaces
        .lock()
        .unwrap() // Can use unwrap() because Mutex will not error unless other thread panicks
        .values()
        .into_iter()
        .map(|x| {
            x.lock().unwrap().set_config(WgIfCfg {
                listen_port: todo!(),
                privkey: todo!(),
            });
        });

    Some(Json(vec![]))
}

#[get("/interface/<id>")]
pub(crate) async fn get_iface(id: String) -> Option<Json<InterfaceConfig>> {
    None
}

#[put("/interface/<id>", format = "json", data = "<ifcfg>")]
pub(crate) async fn update_iface(id: String, ifcfg: Json<InterfaceConfig>) -> Option<Json<String>> {
    None
}

#[delete("/interface/<id>")]
pub(crate) async fn delete_iface(id: String) -> Option<String> {
    None
}

// Interface startup/shutdown
#[get("/interface/<id>/status")]
pub(crate) async fn get_status(id: String) -> Option<Json<InterfaceStatus>> {
    None
}

#[put("/interface/<id>/status", format = "json", data = "<status>")]
pub(crate) async fn put_status(id: String, status: Json<InterfaceStatus>) -> Json<InterfaceStatus> {
    status
}
