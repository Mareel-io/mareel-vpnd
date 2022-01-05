use std::fs;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub api: Api,
    pub wireguard: WireguardConfig,
}

#[derive(Deserialize)]
pub struct Api {
    pub listen: Option<String>,
    pub port: Option<u16>,
    pub apikey: String,
}

#[derive(Deserialize)]
pub struct WireguardConfig {
    pub userspace: Option<String>,
}

pub const WG_USERSPACE_IMPL: &str = "./boringtun";

pub fn read_config(cfgpath: &str, panic_on_notfound: bool) -> Config {
    let mut wgpath = std::env::current_exe().unwrap();
    wgpath.pop();
    wgpath.push(WG_USERSPACE_IMPL);
    match fs::read_to_string(cfgpath) {
        Ok(x) => toml::from_str(&x).expect("Invalid config file!"),
        Err(_) => match panic_on_notfound {
            true => panic!("Config file not found!"),
            false => Config {
                api: Api {
                    listen: None,
                    port: None,
                    apikey: "crowbar".to_string(),
                },
                wireguard: WireguardConfig {
                    userspace: Some(wgpath.to_str().unwrap().to_string()),
                },
            },
        },
    }
}
