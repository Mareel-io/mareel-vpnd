use rocket::serde::json::Json;

use crate::api::common::ApiResponse;

use super::PeerConfig;

#[post("/interface/<if_id>/peer", format = "json", data = "<peercfg>")]
pub(crate) async fn create_peer(
    if_id: String,
    peercfg: Json<PeerConfig>,
) -> Option<Json<ApiResponse<String>>> {
    let ret: ApiResponse<String> = ApiResponse {
        status: Some("ok".to_string()),
        data: None,
    };
    Some(Json(ret))
}

#[get("/interface/<if_id>/peer")]
pub(crate) async fn get_peers(if_id: String) -> Option<Json<Vec<PeerConfig>>> {
    Some(Json(vec![]))
}

#[get("/interface/<if_id>/peer/<pubk>")]
pub(crate) async fn get_peer(if_id: String, pubk: String) -> Option<Json<PeerConfig>> {
    None
}

#[put("/interface/<if_id>/peer/<pubk>", format = "json", data = "<peercfg>")]
pub(crate) async fn update_peer(
    if_id: String,
    pubk: String,
    peercfg: Json<PeerConfig>,
) -> Option<Json<String>> {
    None
}

#[delete("/interface/<if_id>/peer/<pubk>")]
pub(crate) async fn delete_peer(if_id: String, pubk: String) -> Option<String> {
    None
}
