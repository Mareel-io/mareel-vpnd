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

use std::fs;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub api: Api,
    pub wireguard: Option<WireguardConfig>,
    pub cnc: Option<CnC>,
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
    pub use_kernel: Option<bool>,
}

#[derive(Deserialize)]
pub struct CnC {
    pub cnc_url: String,
    pub max_attempts: Option<usize>,
}

const WG_USERSPACE_IMPL: &str = "./boringtun";

fn get_wgpath() -> String {
    let mut wgpath = std::env::current_exe().unwrap();
    wgpath.pop();
    wgpath.push(WG_USERSPACE_IMPL);
    wgpath.to_str().unwrap().to_string()
}

fn platform_default_use_wgkernel() -> bool {
    #[cfg(target_os = "linux")]
    return true;
    #[cfg(target_os = "windows")]
    return true;
    // Kernel implementation is not exist in this platform
    #[allow(unreachable_code)]
    false
}

fn get_default_config() -> Config {
    Config {
        api: Api {
            listen: None,
            port: None,
            apikey: "$argon2id$v=19$m=4096,t=3,p=1$mtHixgMiWZiIwrahCxk/rA$3ci+tSnCgVE52OCVaJHoJF3pjPhb2kt4l6l+jHi6Kuw".to_string(),
        },
        wireguard: Some(WireguardConfig {
            userspace: Some(get_wgpath()),
            use_kernel: Some(platform_default_use_wgkernel()),
        }),
        cnc: None,
    }
}

fn parse_toml(tomlstr: &str) -> Config {
    let mut cfg: Config = toml::from_str(tomlstr).expect("Invalid config file");

    if cfg.wireguard.is_none() {
        cfg.wireguard = get_default_config().wireguard;
    } else {
        if cfg.wireguard.as_ref().unwrap().userspace.is_none() {
            cfg.wireguard = Some(WireguardConfig {
                userspace: get_default_config().wireguard.unwrap().userspace,
                ..cfg.wireguard.unwrap()
            });
        }

        if cfg.wireguard.as_ref().unwrap().use_kernel.is_none() {
            cfg.wireguard = Some(WireguardConfig {
                use_kernel: get_default_config().wireguard.unwrap().use_kernel,
                ..cfg.wireguard.unwrap()
            });
        }
    }

    cfg
}

pub fn read_config(cfgpath: &str, panic_on_notfound: bool) -> Config {
    match fs::read_to_string(cfgpath) {
        Ok(x) => parse_toml(&x),
        Err(_) => match panic_on_notfound {
            true => panic!("Config file not found!"),
            false => get_default_config(),
        },
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_baseline_config() {
        super::parse_toml(
            r##"
        [api]
        apikey = "crowbar"
        "##,
        );
    }

    #[test]
    fn test_cnc_config() {
        let res = super::parse_toml(
            r##"
        [api]
        apikey = "crowbar"
        [cnc]
        cnc_url = "https://example.com"
        "##,
        );

        assert_eq!(res.cnc.unwrap().cnc_url, "https://example.com");
    }
}
