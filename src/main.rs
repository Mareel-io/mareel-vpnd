use std::net::IpAddr;
use std::str::FromStr;

use rocket::config::Config;
use rocket::fairing::AdHoc;
use rocket::tokio::sync::mpsc::Receiver;

#[cfg(not(target_os = "windows"))]
use rocket::tokio::runtime::Runtime;

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate lazy_static;

mod api;
mod vpnctrl;

#[cfg(target_os = "windows")]
mod winsvc;

//#[launch]
pub(crate) async fn launch(shdn: Option<Receiver<()>>) -> Result<(), rocket::Error> {
    let cfg = Config {
        address: IpAddr::from_str("0.0.0.0").unwrap(),
        port: 8080,
        ..Default::default()
    };

    rocket::custom(cfg)
        .attach(api::stage())
        .attach(AdHoc::on_liftoff("Shutdown", move |rocket| {
            Box::pin(async move {
                let shutdown = rocket.shutdown();
                rocket::tokio::spawn(async move {
                    if let Some(mut c) = shdn {
                        c.recv().await;
                        shutdown.notify();
                    }
                });
            })
        }))
        .ignite()
        .await?
        .launch()
        .await
}

#[cfg(not(target_os = "windows"))]
fn main() {
    Runtime::new().unwrap().block_on(launch(None)).unwrap();
}

#[cfg(target_os = "windows")]
fn main() -> windows_service::Result<()> {
    winsvc::run()
}
