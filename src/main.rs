#[macro_use] extern crate rocket;
#[macro_use] extern crate lazy_static;

mod api;
mod vpnctrl;

#[launch]
fn rocket() -> _ {
    rocket::build().attach(
        api::stage()
    )
}