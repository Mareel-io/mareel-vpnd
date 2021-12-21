use std::process::Command;

use super::super::common::{PlatformRoute, PlatformError};
use crate::vpnctrl::error::{InternalError, VpnctrlError};
use wireguard_control::backends::userspace::resolve_tun;

use wireguard_control::InterfaceName;

pub struct Route {}

impl PlatformRoute for Route {
    fn new(_fwmark: u32) -> Result<Self, PlatformError>
    where
        Self: Sized,
    {
        Ok(Self {})
    }

    fn init(&mut self) -> Result<(), Box<dyn VpnctrlError>> {
        Ok(())
    }

    fn add_route(&mut self, ifname: &String, ip: &String) -> Result<(), Box<dyn VpnctrlError>> {
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
            .arg(ip)
            .arg("-interface")
            .arg(real_ifname)
            .output()
        {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(InternalError::new(e.to_string()))),
        }
    }

    fn remove_route(&mut self, ifname: &String, ip: &String) -> Result<(), Box<dyn VpnctrlError>> {
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
            .arg(ip)
            .arg("-interface")
            .arg(real_ifname)
            .output()
        {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(InternalError::new(e.to_string()))),
        }
    }

    fn add_route_bypass(&mut self, _address: &String) -> Result<(), Box<dyn VpnctrlError>> {
        Ok(())
    }

    fn remove_default_route(&mut self) -> Result<(), Box<dyn VpnctrlError>> {
        // Back up default route
        Ok(())
    }

    fn restore_default_route(&mut self) -> Result<(), Box<dyn VpnctrlError>> {
        Ok(())
    }
}

impl Route {
    fn get_real_ifname(alias: &String) -> Result<String, PlatformError> {
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
}
