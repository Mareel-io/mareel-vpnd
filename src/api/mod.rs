#[macro_use] extern crate rocket;

use rocket::serde::{Serialize, Deserialize, json::Json};

pub(crate) mod interface;

struct InterfaceConfig

#[post("/interface", format="json", data="ifcfg")]
async fn create_iface(ifcfg: Json<>) -> io::Result<Json<>>