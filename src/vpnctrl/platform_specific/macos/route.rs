use std::process::Command;

use super::super::common::{PlatformError, PlatformRoute};
use crate::vpnctrl::error::{InternalError, VpnctrlError};
use wireguard_control::backends::userspace::resolve_tun;

use wireguard_control::InterfaceName;

pub struct Route {
    default_gw: (String, String),
    default_route_removed: bool,
}

impl PlatformRoute for Route {
    fn new(_fwmark: u32) -> Result<Self, PlatformError>
    where
        Self: Sized,
    {
        Ok(Self {
            default_gw: ("".to_string(), "".to_string()),
            default_route_removed: false,
        })
    }

    fn init(&mut self) -> Result<(), Box<dyn VpnctrlError>> {
        // TODO: parse and store default route
        Ok(())
    }

    fn add_route(&mut self, ifname: &str, cidr: &str) -> Result<(), Box<dyn VpnctrlError>> {
        let real_ifname = match Self::get_real_ifname(ifname) {
            Ok(x) => x,
            Err(e) => return Err(Box::new(InternalError::new(e.to_string()))),
        };

        // TODO: Support IPv6!
        match Command::new("route")
            .arg("-q")
            .arg("-n")
            .arg("add")
            .arg("-inet")
            .arg(cidr)
            .arg("-interface")
            .arg(real_ifname)
            .output()
        {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(InternalError::new(e.to_string()))),
        }
    }

    fn remove_route(&mut self, ifname: &str, cidr: &str) -> Result<(), Box<dyn VpnctrlError>> {
        let real_ifname = match Self::get_real_ifname(ifname) {
            Ok(x) => x,
            Err(e) => return Err(Box::new(InternalError::new(e.to_string()))),
        };

        // TODO: Support IPv6!
        match Command::new("route")
            .arg("-q")
            .arg("-n")
            .arg("delete")
            .arg("-inet")
            .arg(cidr)
            .arg("-interface")
            .arg(real_ifname)
            .output()
        {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(InternalError::new(e.to_string()))),
        }
    }

    fn add_route_bypass(&mut self, address: &str) -> Result<(), Box<dyn VpnctrlError>> {
        // TODO: Support IPv6!
        match Command::new("route")
            .arg("-q")
            .arg("-n")
            .arg("add")
            .arg("-inet")
            .arg(address)
            .arg(&self.default_gw.0)
            .arg(&self.default_gw.1)
            .output()
        {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(InternalError::new(e.to_string()))),
        }
    }

    fn backup_default_route(&mut self) -> Result<(), Box<dyn VpnctrlError>> {
        // Back up default route
        self.default_gw = match Self::get_default_node_cmd("-inet") {
            Ok(x) => x,
            Err(e) => return Err(Box::new(InternalError::new(e.to_string()))),
        };

        self.default_route_removed = false;

        Ok(())
    }

    fn remove_default_route(&mut self) -> Result<(), Box<dyn VpnctrlError>> {
        if self.default_route_removed {
            return Ok(());
        }

        self.default_route_removed = true;

        // TODO: Support IPv6!
        match Command::new("route")
            .arg("-q")
            .arg("-n")
            .arg("delete")
            .arg("-inet")
            .arg("default")
            .output()
        {
            Ok(_) => Ok(()),
            Err(e) => return Err(Box::new(InternalError::new(e.to_string()))),
        }
    }

    fn restore_default_route(&mut self) -> Result<(), Box<dyn VpnctrlError>> {
        if !self.default_route_removed {
            return Ok(());
        }

        self.default_route_removed = false;
        // TODO: Support IPv6!
        match Command::new("route")
            .arg("-q")
            .arg("-n")
            .arg("delete")
            .arg("-inet")
            .arg("default")
            .output()
        {
            Ok(_) => {}
            Err(e) => return Err(Box::new(InternalError::new(e.to_string()))),
        };

        // TODO: Support IPv6!
        match Command::new("route")
            .arg("-q")
            .arg("-n")
            .arg("add")
            .arg("-inet")
            .arg("default")
            .arg(&self.default_gw.0)
            .arg(&self.default_gw.1)
            .output()
        {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(InternalError::new(e.to_string()))),
        }
    }
}

impl Route {
    fn get_real_ifname(alias: &str) -> Result<String, PlatformError> {
        let lib_ifname: InterfaceName = match alias.parse() {
            Ok(ifname) => ifname,
            Err(_) => {
                return Err(PlatformError::new("Invalid address format".to_string()));
            }
        };

        match resolve_tun(&lib_ifname) {
            Ok(x) => Ok(x),
            Err(_) => {
                return Err(PlatformError::new("What the HELL?".to_string()));
            }
        }
    }

    // Retrieves the node that's currently used to reach 0.0.0.0/0
    // Arguments can be either -inet or -inet6
    fn get_default_node_cmd(if_family: &'static str) -> Result<(String, String), PlatformError> {
        let mut cmd_out = Command::new("route")
            .arg("-n")
            .arg("get")
            .arg(if_family)
            .arg("default")
            .output();

        let stdout = match cmd_out {
            Ok(x) => x.stdout,
            Err(_) => return Err(PlatformError::new("Failed to run route!".to_string())),
        };

        let output = String::from_utf8(stdout).map_err(|e| {
            log::error!("Failed to parse utf-8 bytes from output of netstat - {}", e);
            PlatformError::new("failed to parse utf-8".to_string())
        })?;
        Ok(Self::parse_route(&output))
    }

    fn parse_route(route_output: &str) -> (String, String) {
        for line in route_output.lines() {
            // we're looking for just 2 different lines:
            // interface: utun0
            // gateway: 192.168.3.1
            let tokens: Vec<_> = line.split_whitespace().collect();
            if tokens.len() == 2 {
                match tokens[0].trim() {
                    "interface:" => {
                        return ("-interface".to_string(), tokens[1].to_string());
                    }
                    "gateway:" => {
                        return ("-gateway".to_string(), tokens[1].to_string());
                    }
                    _ => continue,
                }
            }
        }

        ("".to_string(), "".to_string())
    }
}
