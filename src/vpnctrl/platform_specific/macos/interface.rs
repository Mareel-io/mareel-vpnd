use regex::Regex;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::process::Command;
use std::str::FromStr;

use wireguard_control::{
    AllowedIp, Backend, Device, DeviceUpdate, InterfaceName, Key, PeerConfigBuilder,
};

use wireguard_control::backends::userspace::resolve_tun;

use super::super::common::{
    InterfaceStatus, PeerTrafficStat, PlatformError, PlatformInterface, WgIfCfg, WgPeerCfg,
};

use crate::vpnctrl::error::VpnctrlError;

pub struct Interface {
    ifname: InterfaceName,
    real_ifname: String,
    backend: Backend,
    privkey: Key,
    pubkey: Key,
    port: u16,
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
                return Err(VpnctrlError::BadParameterError {
                    msg: "Invalid address format".to_string(),
                });
            }
        };

        match DeviceUpdate::new().apply(&ifname, Backend::Userspace) {
            Ok(_) => (),
            Err(_) => {
                return Err(VpnctrlError::InternalError {
                    msg: "Failed to create interface".to_string(),
                });
            }
        }

        let real_ifname: String = match resolve_tun(&ifname) {
            Ok(x) => x,
            Err(_) => {
                return Err(VpnctrlError::InternalError {
                    msg: "What the HELL?".to_string(),
                });
            }
        };

        Ok(Interface {
            ifname,
            real_ifname,
            backend: Backend::Userspace,
            privkey: Key::zero(),
            pubkey: Key::zero(),
            port: 0,
            peers: HashMap::new(),
            status: InterfaceStatus::Stopped,
        })
    }

    fn set_config(&mut self, cfg: WgIfCfg) -> Result<(), VpnctrlError> {
        self.privkey = match Key::from_base64(cfg.privkey.as_str()) {
            Ok(x) => x,
            Err(_) => {
                return Err(VpnctrlError::BadParameterError {
                    msg: "Invalid privkey format".to_string(),
                })
            }
        };

        let mut update = DeviceUpdate::new().set_private_key(self.privkey.clone());
        update = match cfg.listen_port {
            Some(x) => {
                self.port = x;
                update.set_listen_port(x)
            }
            None => update,
        };

        match update.apply(&self.ifname, self.backend) {
            Ok(_) => (),
            Err(_) => {
                return Err(VpnctrlError::InternalError {
                    msg: "Failed to update interface".to_string(),
                })
            }
        };

        Ok(())
    }

    fn add_peer(&mut self, peer: WgPeerCfg) -> Result<(), VpnctrlError> {
        let pubkey = match Key::from_base64(&peer.pubkey) {
            Ok(x) => x,
            Err(_) => {
                return Err(VpnctrlError::BadParameterError {
                    msg: "Invalid pubkey format".to_string(),
                })
            }
        };

        let psk = match peer.psk {
            Some(ref x) => match Key::from_base64(&x) {
                Ok(x) => Some(x),
                Err(_) => {
                    return Err(VpnctrlError::BadParameterError {
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
            return Err(VpnctrlError::DuplicatedEntryError {
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
                        return Err(VpnctrlError::BadParameterError {
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

        peercfg = peercfg.add_allowed_ips(&allowed_ips.as_slice());

        match DeviceUpdate::new()
            .add_peer(peercfg)
            .apply(&self.ifname, self.backend)
        {
            Ok(_) => (),
            Err(_) => {
                return Err(VpnctrlError::InternalError {
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
                return Err(VpnctrlError::BadParameterError {
                    msg: "Invalid pubkey format".to_string(),
                })
            }
        };

        match self.peers.get(pk.as_slice()) {
            Some(x) => Ok(x.clone()),
            None => Err(VpnctrlError::EntryNotFoundError {
                msg: "Entry not found!".to_string(),
            }),
        }
    }

    fn remove_peer(&mut self, pubkey: &str) -> Result<(), VpnctrlError> {
        let pk = match Key::from_base64(&pubkey) {
            Ok(x) => x,
            Err(_) => {
                return Err(VpnctrlError::BadParameterError {
                    msg: "Invalid pubkey format".to_string(),
                })
            }
        };

        // Lookup the peer
        let mut pubkey_raw: [u8; 32] = [0; 32];
        pubkey_raw.copy_from_slice(pk.as_bytes());
        if self.peers.get(&pubkey_raw).is_none() {
            // Not exist
            return Err(VpnctrlError::EntryNotFoundError {
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
                return Err(VpnctrlError::InternalError {
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
                return Err(VpnctrlError::EntryNotFoundError {
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
        Command::new("ifconfig")
            .arg(&self.real_ifname)
            .arg("mtu")
            .arg("1420")
            .output()
            .expect("Failed to set MTU!");

        Command::new("ifconfig")
            .arg(&self.real_ifname)
            .arg("up")
            .output()
            .expect("Failed to bring up interface!");

        self.status = InterfaceStatus::Running;
        true
    }

    fn down(&mut self) -> bool {
        Command::new("ifconfig")
            .arg(&self.real_ifname)
            .arg("down")
            .output()
            .expect("Failed to bring down interface!");

        self.status = InterfaceStatus::Stopped;
        true
    }

    fn set_ip(&mut self, cidrs: &[String]) -> Result<(), VpnctrlError> {
        let re = Regex::new(r"/.*").unwrap();

        for cidr in cidrs {
            let ip = re.replace_all(cidr, "");
            // TODO: Support IPv6!
            match Command::new("ifconfig")
                .arg(&self.real_ifname)
                .arg("inet")
                .arg(cidr)
                .arg(&*ip)
                .arg("alias")
                .output()
            {
                Ok(_) => {}
                Err(_e) => {
                    return Err(VpnctrlError::InternalError {
                        msg: "Failed to set address".to_string(),
                    })
                }
            };
        }

        Ok(())
    }
}

impl Drop for Interface {
    fn drop(&mut self) -> () {
        self.status = InterfaceStatus::Running;
        let device = match Device::get(&self.ifname, self.backend) {
            Ok(x) => x,
            Err(_) => {
                return ();
            }
        };

        match device.delete() {
            Ok(_) => (),
            Err(_) => (),
        };
    }
}
