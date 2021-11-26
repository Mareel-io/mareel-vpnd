#[macro_use] extern crate rocket;

use rocket::serde::{Serialize, Deserialize, json::Json};

struct InterfaceConfig {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    id: Option<string>
}

#[post("/interface", format="json", data="ifcfg")]
async fn create_iface(ifcfg: Json<>) -> io::Result<Json<>>