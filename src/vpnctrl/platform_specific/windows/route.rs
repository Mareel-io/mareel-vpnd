use super::super::common::PlatformError;
use super::super::common::PlatformRoute;
use crate::vpnctrl::error::{InternalError, VpnctrlError};

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

    fn add_route(&mut self, _ifname: &str, _ip: &str) -> Result<(), Box<dyn VpnctrlError>> {
        Ok(()) // wireguard-nt library does some routing stuff, so just ignore it for now...
    }

    fn remove_route(
        &mut self,
        _ifname: &str,
        _ip: &str,
    ) -> Result<(), Box<dyn VpnctrlError>> {
        Err(Box::new(InternalError::new(
            "Not implemented yet".to_string(),
        )))
    }

    fn add_route_bypass(&mut self, _address: &str) -> Result<(), Box<dyn VpnctrlError>> {
        Ok(())
    }

    fn backup_default_route(&mut self) -> Result<(), Box<dyn VpnctrlError>> {
        Ok(())
    }

    fn remove_default_route(&mut self) -> Result<(), Box<dyn VpnctrlError>> {
        Ok(())
    }

    fn restore_default_route(&mut self) -> Result<(), Box<dyn VpnctrlError>> {
        Ok(())
    }
}
