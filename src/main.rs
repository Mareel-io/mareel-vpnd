use std::net::IpAddr;
use std::str::FromStr;

use clap::Parser;

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

#[derive(clap::Parser)]
#[clap(about, version, author)]
struct Args {
    #[clap(long, short = 'i', value_name = "target")]
    install: Option<String>,

    #[clap(long, short = 'u', value_name = "target")]
    uninstall: Option<String>,

    #[clap(long, value_name = "target")]
    start: Option<String>,

    #[clap(long, value_name = "target")]
    stop: Option<String>,

    #[clap(short = 'c', value_name = "CONFIG")]
    cfg: Option<String>,
}

fn main() -> Result<(), ()> {
    // Do some magic
    let args = Args::parse();

    #[allow(dead_code)]
    match (args.install, args.uninstall, args.start, args.stop) {
        (None, None, None, None) => platform_main(),
        (Some(_method), None, None, None) => {
            #[cfg(not(target_os = "windows"))]
            panic!("Not supported yet!");
            #[cfg(target_os = "windows")]
            winsvc::install().unwrap();
            Ok(())
        }
        (None, Some(_method), None, None) => {
            #[cfg(not(target_os = "windows"))]
            panic!("Not supported yet!");
            #[cfg(target_os = "windows")]
            winsvc::uninstall().unwrap();
            Ok(())
        }
        (None, None, Some(_method), None) => {
            #[cfg(not(target_os = "windows"))]
            panic!("Not supported yet!");
            #[cfg(target_os = "windows")]
            winsvc::start().unwrap();
            Ok(())
        }
        (None, None, None, Some(_method)) => {
            #[cfg(not(target_os = "windows"))]
            panic!("Not supported yet!");
            #[cfg(target_os = "windows")]
            winsvc::stop().unwrap();
            Ok(())
        }
        (_, _, _, _) => panic!("Cannot do those things at the same time!"),
    }
}

#[cfg(not(target_os = "windows"))]
fn platform_main() -> Result<(), ()> {
    match Runtime::new().unwrap().block_on(launch(None)) {
        Ok(_) => Ok(()),
        Err(_) => Err(()), // TODO: Do it properly
    }
}

#[cfg(target_os = "windows")]
fn platform_main() -> Result<(), ()> {
    match winsvc::run() {
        Ok(_) => Ok(()),
        Err(_) => Err(()), // TODO: Do it properly
    }
}
