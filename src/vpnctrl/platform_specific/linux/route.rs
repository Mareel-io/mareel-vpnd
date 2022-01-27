use ipnetwork::IpNetwork;
use wireguard_control::InterfaceName;

use super::super::common::PlatformRoute;
use crate::vpnctrl::error::VpnctrlError;
use crate::vpnctrl::netlink;

pub struct Route {
    fwmark: u32,
}

impl PlatformRoute for Route {
    fn new(fwmark: u32) -> Result<Self, VpnctrlError>
    where
        Self: Sized,
    {
        Ok(Self { fwmark })
    }

    fn init(&mut self) -> Result<(), VpnctrlError> {
        match netlink::add_rule(self.fwmark, self.fwmark, 0x7363) {
            Ok(_) => Ok(()),
            Err(_) => Err(VpnctrlError::Internal {
                msg: "Failed to set routing rule".to_string(),
            }),
        }
    }

    fn add_route(&mut self, ifname: &str, cidr: &str) -> Result<(), VpnctrlError> {
        let wgc_ifname: InterfaceName = match ifname.parse() {
            Ok(ifname) => ifname,
            Err(_) => {
                return Err(VpnctrlError::BadParameter {
                    msg: "Invalid address format".to_string(),
                });
            }
        };

        let ipn: IpNetwork = match cidr.parse() {
            Ok(x) => x,
            Err(_) => {
                return Err(VpnctrlError::BadParameter {
                    msg: "bad cidr".to_string(),
                })
            }
        };
        match netlink::add_route(&wgc_ifname, self.fwmark, ipn) {
            Ok(_) => Ok(()),
            Err(_) => Err(VpnctrlError::Internal {
                msg: "Internal error".to_string(),
            }),
        }
    }

    fn remove_route(&mut self, ifname: &str, cidr: &str) -> Result<(), VpnctrlError> {
        let wgc_ifname: InterfaceName = match ifname.parse() {
            Ok(ifname) => ifname,
            Err(_) => {
                return Err(VpnctrlError::BadParameter {
                    msg: "Invalid address format".to_string(),
                });
            }
        };

        let ipn: IpNetwork = match cidr.parse() {
            Ok(x) => x,
            Err(_) => {
                return Err(VpnctrlError::BadParameter {
                    msg: "bad cidr".to_string(),
                })
            }
        };
        match netlink::del_route(&wgc_ifname, self.fwmark, ipn) {
            Ok(_) => Ok(()),
            Err(_) => Err(VpnctrlError::Internal {
                msg: "Internal error".to_string(),
            }),
        }
    }

    fn add_route_bypass(&mut self, _address: &str) -> Result<(), VpnctrlError> {
        // No need for this. fwmark will handle clutter for us
        Ok(())
    }

    fn remove_route_bypass(&mut self, _address: &str) -> Result<(), VpnctrlError> {
        Ok(())
    }

    fn get_route_bypass(&self) -> Result<Vec<String>, VpnctrlError> {
        Ok(vec![])
    }

    fn backup_default_route(&mut self) -> Result<(), VpnctrlError> {
        // No need for this. fwmark will handle clutter for us
        Ok(())
    }

    fn remove_default_route(&mut self) -> Result<(), VpnctrlError> {
        // No need for this. fwmark will handle clutter for us
        Ok(())
    }

    fn restore_default_route(&mut self) -> Result<(), VpnctrlError> {
        // No need for this. fwmark will handle clutter for us
        Ok(())
    }
}
