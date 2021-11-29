use rocket::serde;
use rocket::serde::json::Json;

use crate::api::common::ApiResponse;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(crate = "rocket::serde")]
pub(crate) struct InterfaceConfig {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub(crate) name: Option<String>,
    pub(crate) private_key: Option<String>,
    pub(crate) public_key: Option<String>,
    pub(crate) listen_port: Option<u16>,
}

#[post("/interface", format="json", data="<ifcfg>")]
pub(crate) async fn create_iface(ifcfg: Json<InterfaceConfig>) -> Option<Json<ApiResponse<String>>> {
    let ret: ApiResponse<String> = ApiResponse {
        status: Some("ok".to_string()),
        data: None,
    };
    Some(Json(ret))
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