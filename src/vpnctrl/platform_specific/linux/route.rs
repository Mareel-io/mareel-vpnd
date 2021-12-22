use ipnetwork::IpNetwork;
use wireguard_control::InterfaceName;

use super::super::common::PlatformError;
use super::super::common::PlatformRoute;
use crate::vpnctrl::error::{BadParameterError, InternalError, VpnctrlError};
use crate::vpnctrl::netlink;

pub struct Route {
    fwmark: u32,
}

impl PlatformRoute for Route {
    fn new(fwmark: u32) -> Result<Self, PlatformError>
    where
        Self: Sized,
    {
        Ok(Self { fwmark })
    }

    fn init(&mut self) -> Result<(), Box<dyn VpnctrlError>> {
        match netlink::add_rule(self.fwmark, self.fwmark, 0x7363) {
            Ok(_) => Ok(()),
            Err(_) => Err(Box::new(InternalError::new(
                "Failed to set routing rule".to_string(),
            ))),
        }
    }

    fn add_route(&mut self, ifname: &String, cidr: &String) -> Result<(), Box<dyn VpnctrlError>> {
        let wgc_ifname: InterfaceName = match ifname.parse() {
            Ok(ifname) => ifname,
            Err(_) => {
                return Err(Box::new(PlatformError::new(
                    "Invalid address format".to_string(),
                )));
            }
        };

        let ipn: IpNetwork = match cidr.parse() {
            Ok(x) => x,
            Err(_) => return Err(Box::new(BadParameterError::new("bad cidr".to_string()))),
        };
        match netlink::add_route(&wgc_ifname, self.fwmark, ipn) {
            Ok(_) => Ok(()),
            Err(_) => Err(Box::new(InternalError::new("Internal error".to_string()))),
        }
    }

    fn remove_route(
        &mut self,
        _ifname: &String,
        _cidr: &String,
    ) -> Result<(), Box<dyn VpnctrlError>> {
        Err(Box::new(InternalError::new(
            "Not implemented yet".to_string(),
        )))
    }

    fn add_route_bypass(&mut self, _address: &String) -> Result<(), Box<dyn VpnctrlError>> {
        // No need for this. fwmark will handle clutter for us
        Ok(())
    }

    fn backup_default_route(&mut self) -> Result<(), Box<dyn VpnctrlError>> {
        // No need for this. fwmark will handle clutter for us
        Ok(())
    }

    fn remove_default_route(&mut self) -> Result<(), Box<dyn VpnctrlError>> {
        // No need for this. fwmark will handle clutter for us
        Ok(())
    }

    fn restore_default_route(&mut self) -> Result<(), Box<dyn VpnctrlError>> {
        // No need for this. fwmark will handle clutter for us
        Ok(())
    }
}
