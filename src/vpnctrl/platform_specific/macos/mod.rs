use wireguard_control::{Backend, DeviceUpdate, InterfaceName, Key};

use super::common::{InterfaceStatus, PlatformError, PlatformInterface};
use crate::vpnctrl::error::VpnctrlError;

pub struct Interface {
    privkey: Key,
    pubkey: Key,
    port: u16,
    status: InterfaceStatus,
}

impl PlatformInterface for Interface {
    fn new(name: &str) -> Result<Self, PlatformError>
    where
        Self: Sized,
    {
        let ifname: InterfaceName = match name.parse().unwrap() {
            Ok(ifname) => ifname,
            Err(_) => {
                return Err(PlatformError::new("Invalid address format".to_string()));
            }
        };

        DeviceUpdate::new().apply(&ifname, Backend::Userspace);

        Ok(Interface {
            privkey: Key::zero(),
            pubkey: Key::zero(),
            port: 0,
            status: InterfaceStatus::Stopped,
        })
    }

    fn set_config(&mut self, cfg: super::common::WgIfCfg) -> Result<(), Box<dyn VpnctrlError>> {
        todo!()
    }

    fn add_peer(&mut self, peer: super::common::WgPeerCfg) -> Result<(), Box<dyn VpnctrlError>> {
        todo!()
    }

    fn get_peers(&self) -> Result<Vec<super::common::WgPeerCfg>, Box<dyn VpnctrlError>> {
        todo!()
    }

    fn get_peer(&self, pubkey: String) -> Result<super::common::WgPeerCfg, Box<dyn VpnctrlError>> {
        todo!()
    }

    fn remove_peer(&mut self, pubkey: String) -> Result<(), Box<dyn VpnctrlError>> {
        todo!()
    }

    fn get_status(&self) -> InterfaceStatus {
        todo!()
    }

    fn up(&self) -> bool {
        todo!()
    }

    fn down(&self) -> bool {
        todo!()
    }
    //
}
