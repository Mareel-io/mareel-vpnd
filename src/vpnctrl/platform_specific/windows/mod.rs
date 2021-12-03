use std::collections::HashMap;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use wireguard_nt::{Adapter, SetInterface, SetPeer};

use crate::vpnctrl::error::{BadParameterError, DuplicatedEntryError, VpnctrlError};

use super::common::{PlatformError, PlatformInterface, WgPeerCfg};

#[cfg(target_arch = "x86_64")]
const DRIVER_DLL_PATH: &str = "./wireguard-nt/bin/amd64/wireguard.dll";
#[cfg(target_arch = "arm")]
const DRIVER_DLL_PATH: &str = "./wireguard-nt/bin/arm/wireguard.dll";
#[cfg(target_arch = "aarch64")]
const DRIVER_DLL_PATH: &str = "./wireguard-nt/bin/arm64/wireguard.dll";
#[cfg(target_arch = "x86")]
const DRIVER_DLL_PATH: &str = "./wireguard-nt/bin/x86/wireguard.dll";

const IF_POOL: &str = "Mareel VPN";

pub struct Interface {
    privkey: [u8; 32],
    pubkey: [u8; 32],
    port: u16,
    iface: Adapter,
    iface_cfg: SetInterface,
    peers: HashMap<[u8; 32], SetPeer>,
}

impl PlatformInterface for Interface {
    fn new(name: &str) -> Result<Interface, PlatformError> {
        let wg = unsafe { wireguard_nt::load_from_path(DRIVER_DLL_PATH) }
            .expect("Failed to load Wireguard DLL");
        let iface = match Interface::create_adapter(wg, name) {
            Ok(iface) => iface,
            Err(e) => return Err(e),
        };

        Ok(Interface {
            privkey: [0; 32],
            pubkey: [0; 32],
            port: 0,
            iface,
            iface_cfg: SetInterface {
                listen_port: None,
                public_key: None,
                private_key: None,
                peers: vec![],
            },
            peers: HashMap::new(),
        })
    }

    fn set_config(&mut self, cfg: super::common::WgIfCfg) -> Result<(), Box<dyn VpnctrlError>> {
        self.privkey.copy_from_slice(
            &(match base64::decode(cfg.privkey) {
                Ok(x) => x,
                Err(_) => {
                    return Err(Box::new(BadParameterError::new(
                        "Invalid privkey format".to_string(),
                    )))
                }
            }),
        );

        self.iface_cfg = SetInterface {
            listen_port: cfg.listen_port,
            public_key: None,
            private_key: Some(self.privkey),
            peers: vec![],
        };

        let ret = match self.iface.set_config(&(self.iface_cfg)) {
            Ok(()) => Ok(()),
            Err(e) => return Err(Box::new(PlatformError::new(e.to_string()))),
        };

        self.port = match cfg.listen_port {
            Some(x) => x,
            None => self.iface.get_config().listen_port,
        };

        self.pubkey = self.iface.get_config().public_key;

        ret
    }

    fn add_peer(&mut self, peer: WgPeerCfg) -> Result<(), Box<dyn VpnctrlError>> {
        let pubkey = match base64::decode(peer.pubkey) {
            Ok(x) => x,
            Err(_) => {
                return Err(Box::new(BadParameterError::new(
                    "Invalid privkey format".to_string(),
                )))
            }
        };

        let mut pubk: [u8; 32] = [0; 32];
        pubk.copy_from_slice(&pubkey);

        match self.peers.get(pubkey.as_slice()) {
            Some(_) => {
                return Err(Box::new(DuplicatedEntryError::new(
                    "Duplicated entry".to_string(),
                )));
            }
            None => {
                self.peers.insert(
                    pubk,
                    SetPeer {
                        public_key: Some(pubk),
                        preshared_key: None,
                        keep_alive: None,
                        endpoint: SocketAddr::from_str("0.0.0.0").unwrap(),
                        allowed_ips: vec![],
                    },
                );
            }
        };

        self.apply_peer_update()
    }

    fn remove_peer(&mut self, pubkey: String) -> Result<(), Box<dyn VpnctrlError>> {
        let pubkey = match base64::decode(pubkey) {
            Ok(x) => x,
            Err(_) => {
                return Err(Box::new(BadParameterError::new(
                    "Invalid privkey format".to_string(),
                )))
            }
        };

        let mut pubk: [u8; 32] = [0; 32];
        pubk.copy_from_slice(&pubkey);

        match self.peers.remove(&pubk) {
            Some(_) => self.apply_peer_update(),
            None => Ok(()),
        }
    }

    fn up(&self) -> bool {
        self.iface.up()
    }

    fn down(&self) -> bool {
        self.iface.down()
    }
}

impl Interface {
    fn create_adapter(wg: Arc<wireguard_nt::dll>, name: &str) -> Result<Adapter, PlatformError> {
        match Adapter::open(wg, name) {
            Ok(iface) => Ok(iface),
            Err((_, wireguard)) => match Adapter::create(wireguard, IF_POOL, name, None) {
                Ok(iface) => Ok(iface),
                Err((e, _)) => Err(PlatformError::new(e.to_string())),
            },
        }
    }

    fn apply_peer_update(&mut self) -> Result<(), Box<dyn VpnctrlError>> {
        // Set up peers
        self.iface_cfg.peers = self.peers.values().cloned().collect();

        match self.iface.set_config(&(self.iface_cfg)) {
            Ok(()) => Ok(()),
            Err(e) => Err(Box::new(PlatformError::new(e.to_string()))),
        }
    }
}
