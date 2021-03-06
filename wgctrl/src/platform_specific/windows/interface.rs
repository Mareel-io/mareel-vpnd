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

use std::collections::HashMap;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use wireguard_nt::{Adapter, SetInterface, SetPeer};

use ipnet::IpNet;

use super::super::common::{InterfaceStatus, PeerTrafficStat, PlatformInterface, WgPeerCfg};

use crate::error::VpnctrlError;

//#[cfg(target_arch = "x86_64")]
//const DRIVER_DLL_PATH: &str = "./wireguard-nt/bin/amd64/wireguard.dll";
//#[cfg(target_arch = "arm")]
//const DRIVER_DLL_PATH: &str = "./wireguard-nt/bin/arm/wireguard.dll";
//#[cfg(target_arch = "aarch64")]
//const DRIVER_DLL_PATH: &str = "./wireguard-nt/bin/arm64/wireguard.dll";
//#[cfg(target_arch = "x86")]
//const DRIVER_DLL_PATH: &str = "./wireguard-nt/bin/x86/wireguard.dll";
const DRIVER_DLL_PATH: &str = "./wireguard.dll";

const IF_POOL: &str = "Mareel VPN";

pub struct Interface {
    privkey: [u8; 32],
    pubkey: [u8; 32],
    port: u16,
    iface: Adapter,
    iface_cfg: SetInterface,
    peers: HashMap<[u8; 32], SetPeer>,
    status: InterfaceStatus,
}

impl PlatformInterface for Interface {
    fn new(name: &str) -> Result<Interface, VpnctrlError> {
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
            status: InterfaceStatus::Stopped,
        })
    }

    fn set_config(&mut self, cfg: super::super::common::WgIfCfg) -> Result<(), VpnctrlError> {
        self.privkey.copy_from_slice(
            &(match base64::decode(cfg.privkey) {
                Ok(x) => x,
                Err(_) => {
                    return Err(VpnctrlError::BadParameter {
                        msg: "Invalid privkey format".to_string(),
                    })
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
            Err(e) => return Err(VpnctrlError::Internal { msg: e.to_string() }),
        };

        self.port = match cfg.listen_port {
            Some(x) => x,
            None => self.iface.get_config().listen_port,
        };

        self.pubkey = self.iface.get_config().public_key;

        ret
    }

    fn add_peer(&mut self, peer: WgPeerCfg) -> Result<(), VpnctrlError> {
        let pubkey = match base64::decode(peer.pubkey) {
            Ok(x) => x,
            Err(_) => {
                return Err(VpnctrlError::BadParameter {
                    msg: "Invalid privkey format".to_string(),
                })
            }
        };

        let psk = match peer.psk {
            Some(x) => match base64::decode(x) {
                Ok(x) => {
                    let mut psk: [u8; 32] = [0; 32];
                    psk.copy_from_slice(&x);
                    Some(psk)
                }
                Err(_) => {
                    return Err(VpnctrlError::BadParameter {
                        msg: "Invalid psk format".to_string(),
                    })
                }
            },
            None => None,
        };

        let endpoint = match peer.endpoint {
            Some(x) => match SocketAddr::from_str(&x) {
                Ok(x) => x,
                Err(_) => {
                    return Err(VpnctrlError::BadParameter {
                        msg: "Invalid endpoint address".to_string(),
                    })
                }
            },
            None => SocketAddr::from_str("0.0.0.0:0").unwrap(),
        };

        let allowed_ips: Vec<IpNet> = peer
            .allowed_ips
            .iter()
            .map(|x| IpNet::from_str(x))
            .filter(|x| x.is_ok())
            .map(|x| x.unwrap())
            .collect();

        let mut pubk: [u8; 32] = [0; 32];
        pubk.copy_from_slice(&pubkey);

        match self.peers.get(pubkey.as_slice()) {
            Some(_) => {
                return Err(VpnctrlError::DuplicatedEntry {
                    msg: "Duplicated entry".to_string(),
                });
            }
            None => {
                self.peers.insert(
                    pubk,
                    SetPeer {
                        public_key: Some(pubk),
                        preshared_key: psk,
                        keep_alive: peer.keep_alive,
                        endpoint,
                        allowed_ips,
                    },
                );
            }
        };

        self.apply_peer_update()
    }

    fn get_platformid(&self) -> Result<String, VpnctrlError> {
        Ok(self.iface.get_luid().to_string())
    }

    fn get_peers(&self) -> Result<Vec<WgPeerCfg>, VpnctrlError> {
        Ok(self
            .peers
            .values()
            .filter(|x| x.public_key.is_some())
            .map(Self::convert_to_wgpeercfg)
            .collect())
    }

    fn get_peer(&self, pubkey: &str) -> Result<WgPeerCfg, VpnctrlError> {
        let pk = match base64::decode(pubkey) {
            Ok(x) => x,
            Err(_) => {
                return Err(VpnctrlError::BadParameter {
                    msg: "Invalid privkey format".to_string(),
                })
            }
        };

        match self.peers.get(pk.as_slice()) {
            Some(x) => Ok(Self::convert_to_wgpeercfg(x)),
            None => Err(VpnctrlError::EntryNotFound {
                msg: "Entry not found!".to_string(),
            }),
        }
    }

    fn remove_peer(&mut self, pubkey: &str) -> Result<(), VpnctrlError> {
        let pubkey = match base64::decode(pubkey) {
            Ok(x) => x,
            Err(_) => {
                return Err(VpnctrlError::BadParameter {
                    msg: "Invalid privkey format".to_string(),
                })
            }
        };

        let mut pubk: [u8; 32] = [0; 32];
        pubk.copy_from_slice(&pubkey);

        match self.peers.remove(&pubk) {
            Some(_) => self.apply_peer_update(),
            None => Ok(()),
        }
    }

    fn get_status(&self) -> InterfaceStatus {
        self.status.clone()
    }

    fn get_trafficstats(&self) -> Result<Vec<PeerTrafficStat>, VpnctrlError> {
        Err(VpnctrlError::Internal {
            msg: "Not implemented yet".to_string(),
        })
    }

    fn up(&mut self) -> bool {
        self.status = InterfaceStatus::Running;
        self.iface.up()
    }

    fn down(&mut self) -> bool {
        self.status = InterfaceStatus::Stopped;
        self.iface.down()
    }

    fn set_ip(&mut self, ips: &[String]) -> Result<(), VpnctrlError> {
        let iplist: Vec<IpNet> = ips
            .iter()
            .map(|x| IpNet::from_str(x))
            .filter(|x| x.is_ok())
            .map(|x| x.unwrap())
            .collect();

        match self.iface.set_default_route(&iplist, &self.iface_cfg) {
            Ok(()) => Ok(()),
            Err(e) => Err(VpnctrlError::Internal { msg: e.to_string() }),
        }
    }
}

impl Interface {
    fn create_adapter(wg: Arc<wireguard_nt::dll>, name: &str) -> Result<Adapter, VpnctrlError> {
        match Adapter::open(wg, name) {
            Ok(iface) => Ok(iface),
            Err((_, wireguard)) => match Adapter::create(wireguard, IF_POOL, name, None) {
                Ok(iface) => Ok(iface),
                Err((e, _)) => Err(VpnctrlError::Internal { msg: e.to_string() }),
            },
        }
    }

    fn apply_peer_update(&mut self) -> Result<(), VpnctrlError> {
        // Set up peers
        self.iface_cfg.peers = self.peers.values().cloned().collect();

        match self.iface.set_config(&(self.iface_cfg)) {
            Ok(()) => Ok(()),
            Err(e) => Err(VpnctrlError::Internal { msg: e.to_string() }),
        }
    }

    fn convert_to_wgpeercfg(peer: &SetPeer) -> WgPeerCfg {
        WgPeerCfg {
            pubkey: base64::encode(peer.public_key.unwrap()),
            psk: peer.preshared_key.map(base64::encode),
            endpoint: Some(peer.endpoint.to_string()),
            allowed_ips: peer
                .allowed_ips
                .clone()
                .into_iter()
                .map(|x| x.to_string())
                .collect(),
            keep_alive: peer.keep_alive,
        }
    }
}
