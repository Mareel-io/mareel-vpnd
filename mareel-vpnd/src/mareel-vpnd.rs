/*
 * SPDX-FileCopyrightText: 2022 Empo Inc.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful, but
 * WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
 * General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

use std::net::IpAddr;
use std::process::Command;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

#[cfg(target_family = "unix")]
use std::os::unix::prelude::CommandExt;

use clap::Parser;

use config::read_config;
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
mod util;

use util::svcman::{svc_install, svc_start, svc_stop, svc_uninstall};

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
        None => cfg
            .wireguard
            .unwrap()
            .userspace
            .expect("Wireguard path is not supplied!"),
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
