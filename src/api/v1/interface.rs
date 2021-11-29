use rocket::{http::Status, serde};
use rocket::serde::json::Json;

use crate::api::common::{ApiError, ApiResponse};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(crate = "rocket::serde")]
pub(crate) struct InterfaceConfig {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub(crate) name: Option<String>,
    pub(crate) private_key: Option<String>,
    pub(crate) public_key: Option<String>,
    pub(crate) listen_port: Option<u16>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(crate = "rocket::serde")]
pub(crate) struct InterfaceStatus {
    pub(crate) status: String,
}

#[post("/interface", format="json", data="<ifcfg>")]
pub(crate) async fn create_iface(ifcfg: Json<InterfaceConfig>) -> (Status, Result<Json<ApiResponse<String>>, Json<ApiError>>) {
    if ifcfg.name == None || ifcfg.private_key == None {
        return (Status::BadRequest, Err(Json(ApiError {
            code: -1,
            msg: "Cannot create interface without its name nor private key".to_string(),
        })));
    }

    let ret: ApiResponse<String> = ApiResponse {
        status: Some("ok".to_string()),
        data: None,
    };
    (Status::Ok, Ok(Json(ret)))
}

#[get("/interface")]
pub(crate) async fn get_ifaces() -> Option<Json<Vec<InterfaceConfig>>> {
    Some(Json(vec![]))
}

#[get("/interface/<id>")]
pub(crate) async fn get_iface(id: String) -> Option<Json<InterfaceConfig>> {
    None
}

#[put("/interface/<id>", format="json", data="<ifcfg>")]
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

#[put("/interface/<id>/status", format="json", data="<status>")]
pub(crate) async fn put_status(id: String, status: Json<InterfaceStatus>) -> Json<InterfaceStatus> {
    status
}