use std::collections::HashMap;

use super::common::{InterfaceStatus, PlatformError, PlatformInterface, WgIfCfg, WgPeerCfg};
use crate::vpnctrl::error::VpnctrlError;

use wireguard_control::{Backend, DeviceUpdate, InterfaceName, Key, PeerConfigBuilder};

use super::common::{InterfaceStatus, PlatformError, PlatformInterface, WgIfCfg, WgPeerCfg};
use crate::vpnctrl::error::{
    BadParameterError, DuplicatedEntryError, EntryNotFoundError, VpnctrlError,
};

pub struct Interface {
    ifname: InterfaceName,
    backend: Backend,
    privkey: Key,
    pubkey: Key,
    port: u16,
    peers: HashMap<[u8; 32], WgPeerCfg>,
    status: InterfaceStatus,
}

impl PlatformInterface for Interface {
    fn new(name: &str) -> Result<Self, PlatformError>
    where
        Self: Sized,
    {
        let ifname: InterfaceName = match name.parse() {
            Ok(ifname) => ifname,
            Err(_) => {
                return Err(PlatformError::new("Invalid address format".to_string()));
            }
        };

        DeviceUpdate::new().apply(&ifname, Backend::Kernel);

        Ok(Interface {
            ifname,
            backend: Backend::Kernel,
            privkey: Key::zero(),
            pubkey: Key::zero(),
            port: 0,
            peers: HashMap::new(),
            status: InterfaceStatus::Stopped,
        })
    }

    fn set_config(&mut self, cfg: WgIfCfg) -> Result<(), Box<dyn VpnctrlError>> {
        self.privkey = match Key::from_base64(cfg.privkey.as_str()) {
            Ok(x) => x,
            Err(_) => {
                return Err(Box::new(BadParameterError::new(
                    "Invalid privkey format".to_string(),
                )))
            }
        };

        let mut update = DeviceUpdate::new().set_private_key(self.privkey.clone());
        update = match cfg.listen_port {
            Some(x) => update.set_listen_port(x),
            None => update,
        };

        update.apply(&self.ifname, self.backend);

        Ok(())
    }

    fn add_peer(&mut self, peer: WgPeerCfg) -> Result<(), Box<dyn VpnctrlError>> {
        let pubkey = match Key::from_base64(&peer.pubkey) {
            Ok(x) => x,
            Err(_) => {
                return Err(Box::new(BadParameterError::new(
                    "Invalid pubkey format".to_string(),
                )))
            }
        };

        let psk = match peer.psk {
            Some(ref x) => match Key::from_base64(&x) {
                Ok(x) => Some(x),
                Err(_) => {
                    return Err(Box::new(BadParameterError::new(
                        "Invalid psk format".to_string(),
                    )))
                }
            },
            None => None,
        };

        // Lookup the peer
        let mut pubkey_raw: [u8; 32] = [0; 32];
        pubkey_raw.copy_from_slice(pubkey.as_bytes());
        if self.peers.get(&pubkey_raw).is_some() {
            // Collision!!
            return Err(Box::new(DuplicatedEntryError::new(
                "Duplicated peer".to_string(),
            )));
        }

        let mut peercfg = PeerConfigBuilder::new(&pubkey);
        peercfg = match psk {
            Some(x) => peercfg.set_preshared_key(x),
            None => peercfg,
        };

        //
        DeviceUpdate::new()
            .add_peer(peercfg)
            .apply(&self.ifname, self.backend);

        // Add the peer
        self.peers.insert(pubkey_raw, peer);
        Ok(())
    }

    fn get_peers(&self) -> Result<Vec<WgPeerCfg>, Box<dyn VpnctrlError>> {
        Ok(self.peers.values().cloned().collect())
    }

    fn get_peer(&self, pubkey: String) -> Result<WgPeerCfg, Box<dyn VpnctrlError>> {
        let pk = match base64::decode(pubkey) {
            Ok(x) => x,
            Err(_) => {
                return Err(Box::new(BadParameterError::new(
                    "Invalid privkey format".to_string(),
                )))
            }
        };

        match self.peers.get(pk.as_slice()) {
            Some(x) => Ok(x.clone()),
            None => Err(Box::new(EntryNotFoundError::new(
                "Entry not found!".to_string(),
            ))),
        }
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
}
