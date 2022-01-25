use std::fs::File;
use std::io::Write;
use std::process::Command;

use argon2::password_hash::{rand_core::OsRng, PasswordHasher, SaltString};
use argon2::Argon2;
use clap::Parser;
use serde::Serialize;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref ARGS: Args = Args::parse();
}

#[derive(clap::Parser)]
#[clap(about, version, author)]
struct Args {
    #[clap(long, short = 'c', value_name = "CONFIG")]
    config: String,

    #[clap(long, short = 't', value_name = "password")]
    token: Option<String>,

    #[clap(long, short = 'p', value_name = "port")]
    port: Option<u16>,

    #[clap(long, short = 'r', value_name = "method")]
    reload: Option<String>,
}

#[derive(Serialize)]
struct Config {
    pub api: Api,
}

#[derive(Serialize)]
struct Api {
    pub listen: String,
    pub port: Option<u16>,
    pub apikey: String,
}

fn main() -> Result<(), ()> {
    // Generate salt
    let salt = SaltString::generate(&mut OsRng);
    let token = match &ARGS.token {
        Some(x) => x,
        None => "crowbar",
    };

    let argon2 = Argon2::default();
    let token_hash = argon2
        .hash_password(token.as_bytes(), &salt)
        .unwrap()
        .to_string();

    let cfg = Config {
        api: Api {
            listen: "127.0.0.1".to_string(),
            port: ARGS.port,
            apikey: token_hash,
        },
    };

    let cfgstr = toml::to_string(&cfg).unwrap();

    let mut cfgfile = File::create(&ARGS.config).expect("Failed to create config file!");
    cfgfile
        .write_all(cfgstr.as_bytes())
        .expect("Failed to write to config file!");
    drop(cfgfile);

    if ARGS.reload.is_some() {
        // Launch vpnd and set up the daemon!

        let method = ARGS.reload.as_ref().unwrap();
        let mut vpnd_path = ::std::env::current_exe().unwrap();
        vpnd_path.pop();
        #[cfg(target_os = "windows")]
        vpnd_path.push("mareel-vpnd.exe");
        #[cfg(not(target_os = "windows"))]
        vpnd_path.push("mareel-vpnd");

        Command::new(&vpnd_path)
            .arg("--uninstall")
            .arg(method)
            .output()
            .expect("Failed to uninstall daemon!");
        Command::new(&vpnd_path)
            .arg("--config")
            .arg(&ARGS.config)
            .arg("--install")
            .arg(method)
            .arg("--start")
            .arg(method)
            .output()
            .expect("Failed to reload daemon!");
    }

    Ok(())
}
