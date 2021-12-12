use std::net::IpAddr;
use std::str::FromStr;

use clap::Parser;

use config::read_config;
use rocket::config::Config;
use rocket::fairing::AdHoc;
use rocket::tokio::sync::mpsc::Receiver;

use rocket::tokio::runtime::Runtime;

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate lazy_static;

mod api;
mod config;
mod svc;
mod vpnctrl;

lazy_static! {
    static ref ARGS: Args = Args::parse();
}

//#[launch]
pub(crate) async fn launch(
    shdn: Option<Receiver<()>>,
    daemon_cfg: &config::Config,
) -> Result<(), rocket::Error> {
    let listen = match &daemon_cfg.api.listen {
        Some(x) => x,
        None => "127.0.0.1",
    };

    let port = match &daemon_cfg.api.port {
        Some(x) => x.to_owned(),
        None => 8080,
    };

    let cfg = Config {
        address: IpAddr::from_str(&listen).unwrap(),
        port,
        ..Default::default()
    };

    rocket::custom(cfg)
        // TODO: FIXME
        .attach(api::stage(&daemon_cfg.api.apikey))
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

fn launcher(shdn: Option<Receiver<()>>) -> Result<(), ()> {
    // Read config file
    let cfgpath = match &ARGS.config {
        Some(x) => x,
        None => "./mareel-vpnd.toml",
    };

    let cfg = read_config(cfgpath, ARGS.config.is_some());

    match Runtime::new().unwrap().block_on(launch(shdn, &cfg)) {
        Ok(_) => Ok(()),
        Err(_) => Err(()), // TODO: Do it properly
    }
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

    #[clap(long, short = 'c', value_name = "CONFIG")]
    config: Option<String>,
}

fn main() -> Result<(), ()> {
    // Do some magic
    let args = &ARGS;

    match (&args.install, &args.uninstall, &args.start, &args.stop) {
        (None, None, None, None) => platform_main(),
        (Some(method), None, None, None) => {
            #[cfg(target_os = "linux")]
            {
                match method.as_str() {
                    "systemd" => svc::systemd::install(&args.config).unwrap(),
                    _ => panic!("Not supported feature: {}", method),
                };
                return Ok(());
            }
            #[cfg(target_os = "windows")]
            {
                match method.as_str() {
                    "winsvc" => svc::winsvc::install().unwrap(),
                    _ => panic!("Not supported feature: {}", method),
                };
                return Ok(());
            }
            #[allow(unreachable_code)]
            {
                panic!("Not supported yet!");
            }
        }
        (None, Some(method), None, None) => {
            #[cfg(target_os = "linux")]
            {
                match method.as_str() {
                    "systemd" => svc::systemd::uninstall().unwrap(),
                    _ => panic!("Not supported feature: {}", method),
                };
                return Ok(());
            }
            #[cfg(target_os = "windows")]
            {
                match method.as_str() {
                    "winsvc" => svc::winsvc::uninstall().unwrap(),
                    _ => panic!("Not supported feature: {}", method),
                };
                return Ok(());
            }
            #[allow(unreachable_code)]
            {
                panic!("Not supported yet!");
            }
        }
        (None, None, Some(method), None) => {
            #[cfg(target_os = "linux")]
            {
                match method.as_str() {
                    "systemd" => svc::systemd::start().unwrap(),
                    _ => panic!("Not supported feature: {}", method),
                };
                return Ok(());
            }
            #[cfg(target_os = "windows")]
            {
                match method.as_str() {
                    "winsvc" => svc::winsvc::start().unwrap(),
                    _ => panic!("Not supported feature: {}", method),
                };
                return Ok(());
            }
            #[allow(unreachable_code)]
            {
                panic!("Not supported yet!");
            }
        }
        (None, None, None, Some(method)) => {
            #[cfg(target_os = "linux")]
            {
                match method.as_str() {
                    "systemd" => svc::systemd::stop().unwrap(),
                    _ => panic!("Not supported feature: {}", method),
                };
                return Ok(());
            }
            #[cfg(target_os = "windows")]
            {
                match method.as_str() {
                    "winsvc" => svc::winsvc::stop().unwrap(),
                    _ => panic!("Not supported feature: {}", method),
                };
                return Ok(());
            }
            #[allow(unreachable_code)]
            {
                panic!("Not supported yet!");
            }
        }
        (_, _, _, _) => panic!("Cannot do those things at the same time!"),
    }
}

#[cfg(not(target_os = "windows"))]
fn platform_main() -> Result<(), ()> {
    launcher(None)
}

#[cfg(target_os = "windows")]
fn platform_main() -> Result<(), ()> {
    match svc::winsvc::run() {
        Ok(_) => Ok(()),
        Err(_) => Err(()), // TODO: Do it properly
    }
}
