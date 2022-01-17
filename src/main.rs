use std::net::IpAddr;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

#[cfg(target_family = "unix")]
use std::os::unix::prelude::CommandExt;

use clap::Parser;

use config::{read_config, WG_USERSPACE_IMPL};
use rocket::config::Config;
use rocket::fairing::AdHoc;
use rocket::tokio::sync::mpsc::Receiver;

use rocket::tokio::runtime::Runtime;

use prometheus::Registry;

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
    static ref PROM_REGISTRY: Arc<Mutex<Registry>> = Arc::new(Mutex::new(Registry::new()));
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
        None => 29539,
    };

    let cfg = Config {
        address: IpAddr::from_str(listen).unwrap(),
        port,
        ..Default::default()
    };

    // Launch monitoring thread for the daemon

    rocket::custom(cfg)
        // TODO: FIXME
        .attach(api::stage(
            &daemon_cfg.api.apikey,
            Arc::clone(&PROM_REGISTRY),
        ))
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

    #[clap(long, value_name = "target")]
    restart: Option<String>,

    #[clap(long, short = 'c', value_name = "CONFIG")]
    config: Option<String>,

    #[clap(long)]
    foreground: bool,

    #[clap(long, value_name = "wireguard userspace daemon")]
    wireguard: Option<String>,
}

fn svc_install(method: &str, config: &Option<String>) -> Result<(), ()> {
    #[cfg(target_os = "linux")]
    {
        match method {
            "systemd" => svc::systemd::install(config).unwrap(),
            _ => panic!("Not supported feature: {}", method),
        };
        return Ok(());
    }
    #[cfg(target_os = "windows")]
    {
        match method {
            "winsvc" => svc::winsvc::install(config).unwrap(),
            _ => panic!("Not supported feature: {}", method),
        };
        return Ok(());
    }
    #[cfg(target_os = "macos")]
    {
        match method {
            "launchd" => svc::launchd::install(config).unwrap(),
            _ => panic!("Not supported feature: {}", method),
        };
        return Ok(());
    }
    #[allow(unreachable_code)]
    {
        panic!("Not supported yet!");
    }
}

fn svc_uninstall(method: &str) -> Result<(), ()> {
    #[cfg(target_os = "linux")]
    {
        match method {
            "systemd" => svc::systemd::uninstall().unwrap(),
            _ => panic!("Not supported feature: {}", method),
        };
        return Ok(());
    }
    #[cfg(target_os = "windows")]
    {
        match method {
            "winsvc" => svc::winsvc::uninstall().unwrap(),
            _ => panic!("Not supported feature: {}", method),
        };
        return Ok(());
    }
    #[cfg(target_os = "macos")]
    {
        match method {
            "launchd" => svc::launchd::uninstall().unwrap(),
            _ => panic!("Not supported feature: {}", method),
        };
        return Ok(());
    }
    #[allow(unreachable_code)]
    {
        panic!("Not supported yet!");
    }
}

fn svc_start(method: &str) -> Result<(), ()> {
    #[cfg(target_os = "linux")]
    {
        match method {
            "systemd" => svc::systemd::start().unwrap(),
            _ => panic!("Not supported feature: {}", method),
        };
        return Ok(());
    }
    #[cfg(target_os = "windows")]
    {
        match method {
            "winsvc" => svc::winsvc::start().unwrap(),
            _ => panic!("Not supported feature: {}", method),
        };
        return Ok(());
    }
    #[cfg(target_os = "macos")]
    {
        match method {
            "launchd" => svc::launchd::start().unwrap(),
            _ => panic!("Not supported feature: {}", method),
        };
        return Ok(());
    }
    #[allow(unreachable_code)]
    {
        panic!("Not supported yet!");
    }
}

fn svc_stop(method: &str) -> Result<(), ()> {
    #[cfg(target_os = "linux")]
    {
        match method {
            "systemd" => svc::systemd::stop().unwrap(),
            _ => panic!("Not supported feature: {}", method),
        };
        return Ok(());
    }
    #[cfg(target_os = "windows")]
    {
        match method {
            "winsvc" => svc::winsvc::stop().unwrap(),
            _ => panic!("Not supported feature: {}", method),
        };
        return Ok(());
    }
    #[cfg(target_os = "macos")]
    {
        match method {
            "launchd" => svc::launchd::stop().unwrap(),
            _ => panic!("Not supported feature: {}", method),
        };
        return Ok(());
    }
    #[allow(unreachable_code)]
    {
        panic!("Not supported yet!");
    }
}

fn main() -> Result<(), ()> {
    // Do some magic
    let args = &ARGS;

    // Read config file
    let cfgpath = match &ARGS.config {
        Some(x) => x,
        None => "./mareel-vpnd.toml",
    };

    let cfg = read_config(cfgpath, ARGS.config.is_some());
    let wg_impl = match args.wireguard.clone() {
        Some(x) => x,
        None => {
            //let mut wgpath = std::env::current_exe().unwrap();
            //wgpath.pop();
            //wgpath.push(WG_USERSPACE_IMPL);

            let wgpath = PathBuf::from_str(WG_USERSPACE_IMPL).unwrap();

            cfg.wireguard
                .userspace
                .unwrap_or_else(|| wgpath.to_str().unwrap().to_string())
        }
    };

    fn launch_new(wg_impl: String) -> Result<(), ()> {
        println!("Launching with {}", wg_impl);
        let mut cmd = Command::new(std::env::current_exe().unwrap());
        let cmd_cfg = cmd
            .args(std::env::args().skip(1))
            .env("WG_USERSPACE_IMPLEMENTATION", &wg_impl)
            .env("WG_QUICK_USERSPACE_IMPLEMENTATION", &wg_impl)
            .env("WG_SUDO", "1");

        #[cfg(target_family = "unix")]
        cmd_cfg.exec();
        #[cfg(not(target_family = "unix"))]
        cmd_cfg.status().expect("Failed to re-launch daemon!");
        Ok(())
    }

    match std::env::var("WG_USERSPACE_IMPLEMENTATION") {
        Ok(x) => {
            if x != wg_impl {
                // Re-launch!!
                return launch_new(wg_impl);
            }
        }
        Err(_) => {
            // Re-launch!
            return launch_new(wg_impl);
        }
    }

    if args.foreground {
        println!("Foreground mode requested. Skipping all service stuff.");
        return launcher(None);
    };

    match (
        &args.install,
        &args.uninstall,
        &args.start,
        &args.stop,
        &args.restart,
    ) {
        (None, None, None, None, None) => platform_main(),
        (Some(method), None, None, None, None) => svc_install(method.as_str(), &args.config),
        (None, Some(method), None, None, None) => svc_uninstall(method.as_str()),
        (None, None, Some(method), None, None) => svc_start(method.as_str()),
        (None, None, None, Some(method), None) => svc_stop(method.as_str()),
        (None, None, None, None, Some(method)) => {
            #[allow(unused_must_use)]
            {
                svc_stop(method.as_str());
            }

            svc_start(method.as_str())
        }
        (Some(method), None, Some(method2), None, None) => {
            #[allow(unused_must_use)]
            {
                svc_install(method.as_str(), &args.config);
            }

            svc_start(method2.as_str())
        }
        (None, Some(method2), None, Some(method), None) => {
            #[allow(unused_must_use)]
            {
                svc_stop(method.as_str());
            }

            svc_uninstall(method2.as_str())
        }
        (_, _, _, _, _) => panic!("Cannot do those things at the same time!"),
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
