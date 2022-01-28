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

use std::{collections::HashMap, net::SocketAddr, str::FromStr};

use ipnetwork::IpNetwork;
use wireguard_control::{
    AllowedIp, Backend, Device, DeviceUpdate, InterfaceName, Key, PeerConfigBuilder,
};

use super::super::common::{
    InterfaceStatus, PeerTrafficStat, PlatformInterface, WgIfCfg, WgPeerCfg,
};
use crate::error::VpnctrlError;

use super::super::super::netlink;

pub struct Interface {
    ifname: InterfaceName,
    backend: Backend,
    privkey: Key,
    pubkey: Key,
    port: u16,
    fwmark: u32,
    peers: HashMap<[u8; 32], WgPeerCfg>,
    status: InterfaceStatus,
}

impl PlatformInterface for Interface {
    fn new(name: &str) -> Result<Self, VpnctrlError>
    where
        Self: Sized,
    {
        let ifname: InterfaceName = match name.parse() {
            Ok(ifname) => ifname,
            Err(_) => {
                return Err(VpnctrlError::BadParameter {
                    msg: "Invalid address format".to_string(),
                });
            }
        };

        match DeviceUpdate::new().apply(&ifname, Backend::Kernel) {
            Ok(_) => (),
            Err(e) => {
                return Err(VpnctrlError::Internal { msg: e.to_string() });
            }
        }

        Ok(Interface {
            ifname,
            backend: Backend::Kernel,
            //backend: Backend::Userspace,
            privkey: Key::zero(),
            pubkey: Key::zero(),
            port: 0,
            fwmark: 0,
            peers: HashMap::new(),
            status: InterfaceStatus::Stopped,
        })
    }

    fn set_config(&mut self, cfg: WgIfCfg) -> Result<(), VpnctrlError> {
        self.privkey = match Key::from_base64(cfg.privkey.as_str()) {
            Ok(x) => x,
            Err(_) => {
                return Err(VpnctrlError::BadParameter {
                    msg: "Invalid privkey format".to_string(),
                })
            }
        };
        self.fwmark = cfg.fwmark;

        let mut update = DeviceUpdate::new().set_private_key(self.privkey.clone());
        update = match cfg.listen_port {
            Some(x) => {
                self.port = x;
                update.set_listen_port(x)
            }
            None => update,
        };

        update = update.set_fwmark(cfg.fwmark);

        match update.apply(&self.ifname, self.backend) {
            Ok(_) => Ok(()),
            Err(_) => Err(VpnctrlError::Internal {
                msg: "Failed to update interface".to_string(),
            }),
        }
    }

    fn add_peer(&mut self, peer: WgPeerCfg) -> Result<(), VpnctrlError> {
        let pubkey = match Key::from_base64(&peer.pubkey) {
            Ok(x) => x,
            Err(_) => {
                return Err(VpnctrlError::BadParameter {
                    msg: "Invalid pubkey format".to_string(),
                })
            }
        };

        let psk = match peer.psk {
            Some(ref x) => match Key::from_base64(x) {
                Ok(x) => Some(x),
                Err(_) => {
                    return Err(VpnctrlError::BadParameter {
                        msg: "Invalid psk format".to_string(),
                    })
                }
            },
            None => None,
        };

        // Lookup the peer
        let mut pubkey_raw: [u8; 32] = [0; 32];
        pubkey_raw.copy_from_slice(pubkey.as_bytes());
        if self.peers.get(&pubkey_raw).is_some() {
            // Collision!!
            return Err(VpnctrlError::DuplicatedEntry {
                msg: "Duplicated peer".to_string(),
            });
        }

        let mut peercfg = PeerConfigBuilder::new(&pubkey);
        peercfg = match psk {
            Some(x) => peercfg.set_preshared_key(x),
            None => peercfg,
        };

        peercfg = match peer.endpoint {
            Some(ref x) => {
                let endpt: SocketAddr = match x.parse() {
                    Ok(x) => x,
                    Err(_) => {
                        return Err(VpnctrlError::BadParameter {
                            msg: "Invalid endpoint format".to_string(),
                        })
                    }
                };

                peercfg.set_endpoint(endpt)
            }
            None => peercfg,
        };

        peercfg = match peer.keep_alive {
            Some(x) => peercfg.set_persistent_keepalive_interval(x),
            None => peercfg,
        };

        let allowed_ips: Vec<AllowedIp> = peer
            .allowed_ips
            .iter()
            .map(|x| AllowedIp::from_str(x))
            .filter(|x| x.is_ok())
            .map(|x| x.unwrap())
            .collect();

        peercfg = peercfg.add_allowed_ips(allowed_ips.as_slice());

        match DeviceUpdate::new()
            .add_peer(peercfg)
            .apply(&self.ifname, self.backend)
        {
            Ok(_) => (),
            Err(_) => {
                return Err(VpnctrlError::Internal {
                    msg: "Failed to update interface".to_string(),
                });
            }
        }

        // Add the peer
        self.peers.insert(pubkey_raw, peer);
        Ok(())
    }

    fn get_peers(&self) -> Result<Vec<WgPeerCfg>, VpnctrlError> {
        Ok(self.peers.values().cloned().collect())
    }

    fn get_peer(&self, pubkey: &str) -> Result<WgPeerCfg, VpnctrlError> {
        let pk = match base64::decode(pubkey) {
            Ok(x) => x,
            Err(_) => {
                return Err(VpnctrlError::BadParameter {
                    msg: "Invalid pubkey format".to_string(),
                })
            }
        };

        match self.peers.get(pk.as_slice()) {
            Some(x) => Ok(x.clone()),
            None => Err(VpnctrlError::EntryNotFound {
                msg: "Entry not found!".to_string(),
            }),
        }
    }

    fn remove_peer(&mut self, pubkey: &str) -> Result<(), VpnctrlError> {
        let pk = match Key::from_base64(pubkey) {
            Ok(x) => x,
            Err(_) => {
                return Err(VpnctrlError::BadParameter {
                    msg: "Invalid pubkey format".to_string(),
                })
            }
        };

        // Lookup the peer
        let mut pubkey_raw: [u8; 32] = [0; 32];
        pubkey_raw.copy_from_slice(pk.as_bytes());
        if self.peers.get(&pubkey_raw).is_none() {
            // Not exist
            return Err(VpnctrlError::EntryNotFound {
                msg: "Entry not found".to_string(),
            });
        }

        // Remove peer
        match DeviceUpdate::new()
            .remove_peer_by_key(&pk)
            .apply(&self.ifname, self.backend)
        {
            Ok(_) => (),
            Err(_) => {
                return Err(VpnctrlError::Internal {
                    msg: "Failed to update interface".to_string(),
                })
            }
        };

        self.peers.remove(&pubkey_raw);

        Ok(())
    }

    fn get_status(&self) -> InterfaceStatus {
        self.status.clone()
    }

    fn get_trafficstats(&self) -> Result<Vec<PeerTrafficStat>, VpnctrlError> {
        let dev = match Device::get(&self.ifname, self.backend) {
            Ok(x) => x,
            Err(_) => {
                return Err(VpnctrlError::EntryNotFound {
                    msg: "Entry not found".to_string(),
                })
            }
        };

        Ok(dev
            .peers
            .into_iter()
            .map(|x| PeerTrafficStat {
                pubkey: x.config.public_key.to_base64(),
                rx_bytes: x.stats.rx_bytes,
                tx_bytes: x.stats.tx_bytes,
            })
            .collect())
    }

    fn up(&mut self) -> bool {
        match netlink::set_up(&self.ifname, 1420) {
            Ok(_) => {
                self.status = InterfaceStatus::Running;
                true
            }
            Err(_) => false,
        }
    }

    fn down(&mut self) -> bool {
        match netlink::set_down(&self.ifname) {
            Ok(_) => {
                self.status = InterfaceStatus::Stopped;
                true
            }
            Err(_) => false,
        }
    }

    fn set_ip(&mut self, ips: &[String]) -> Result<(), VpnctrlError> {
        let ipns: Vec<IpNetwork> = ips
            .iter()
            .map(|x| x.parse())
            .filter(|x| x.is_ok())
            .map(|x| x.unwrap())
            .collect();

        for ipn in ipns {
            match netlink::set_addr(&self.ifname, ipn) {
                Ok(_) => {}
                Err(_) => {
                    return Err(VpnctrlError::Internal {
                        msg: "Failed to set address".to_string(),
                    })
                }
            }
        }

        Ok(())
    }
}

impl Drop for Interface {
    fn drop(&mut self) {
        self.status = InterfaceStatus::Running;
        let device = match Device::get(&self.ifname, self.backend) {
            Ok(x) => x,
            Err(_) => {
                return;
            }
        };

        match device.delete() {
            Ok(_) => (),
            Err(_) => {
                println!("Warn: device delete error");
            }
        };
    }
}
