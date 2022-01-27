use rocket::serde::json::Json;
use rocket::{http::Status, State};

use crate::api::common::{ApiResponse, ApiResponseType};
use crate::api::tokenauth::ApiKey;
use crate::vpnctrl::platform_specific::common::PlatformRoute;

use super::types::RouteManagerStore;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(crate = "rocket::serde")]
pub(crate) struct RouteBypass {
    pub addr: String,
}

#[post("/route/bypass", format = "json", data = "<route>")]
pub(crate) async fn create_bypass(
    _apikey: ApiKey,
    rms: &State<RouteManagerStore>,
    route: Json<RouteBypass>,
) -> ApiResponseType<String> {
    let mut rm = rms.route_manager.lock().unwrap();

    match rm.add_route_bypass(&route.addr) {
        Ok(_) => (Status::Ok, ApiResponse::ok("ok".to_string())),
        Err(e) => (
            Status::InternalServerError,
            ApiResponse::err(-1, &e.to_string()),
        ),
    }
}

#[get("/route/bypass")]
pub(crate) async fn get_bypass(
    _apikey: ApiKey,
    rms: &State<RouteManagerStore>,
) -> ApiResponseType<Vec<String>> {
    let rm = rms.route_manager.lock().unwrap();

    match rm.get_route_bypass() {
        Ok(x) => (Status::Ok, ApiResponse::ok(x)),
        Err(e) => (
            Status::InternalServerError,
            ApiResponse::err(-1, &e.to_string()),
        ),
    }
}

#[delete("/route/bypass/<route>")]
pub(crate) async fn delete_bypass(
    _apikey: ApiKey,
    rms: &State<RouteManagerStore>,
    route: String,
) -> ApiResponseType<String> {
    let mut rm = rms.route_manager.lock().unwrap();

    match rm.remove_route_bypass(&route) {
        Ok(_) => (Status::Ok, ApiResponse::ok("ok".to_string())),
        Err(e) => (
            Status::InternalServerError,
            ApiResponse::err(-1, &e.to_string()),
        ),
    }
}
