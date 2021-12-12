use std::{collections::HashMap, net::SocketAddr, str::FromStr};

use ipnetwork::IpNetwork;
use wireguard_control::{
    AllowedIp, Backend, Device, DeviceUpdate, InterfaceName, Key, PeerConfigBuilder,
};

use super::common::{
    InterfaceStatus, PeerTrafficStat, PlatformError, PlatformInterface, WgIfCfg, WgPeerCfg,
};
use crate::vpnctrl::error::{
    BadParameterError, DuplicatedEntryError, EntryNotFoundError, InternalError, VpnctrlError,
};

use super::super::netlink;

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

        match DeviceUpdate::new().apply(&ifname, Backend::Kernel) {
            Ok(_) => (),
            Err(_) => {
                return Err(PlatformError::new("Failed to create interface".to_string()));
            }
        }

        Ok(Interface {
            ifname,
            backend: Backend::Kernel,
            privkey: Key::zero(),
            pubkey: Key::zero(),
            port: 0,
            fwmark: 0,
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
            Ok(_) => (),
            Err(_) => {
                return Err(Box::new(InternalError::new(
                    "Failed to update interface".to_string(),
                )));
            }
        };

        match netlink::add_rule(cfg.fwmark, cfg.fwmark, 0x7363) {
            Ok(_) => Ok(()),
            Err(_) => Err(Box::new(InternalError::new(
                "Failed to set routing rule".to_string(),
            ))),
        }
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

        peercfg = match peer.endpoint {
            Some(ref x) => {
                let endpt: SocketAddr = match x.parse() {
                    Ok(x) => x,
                    Err(_) => {
                        return Err(Box::new(BadParameterError::new(
                            "Invalid endpoint format".to_string(),
                        )))
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
                return Err(Box::new(InternalError::new(
                    "Failed to update interface".to_string(),
                )));
            }
        }

        // Add the peer
        self.peers.insert(pubkey_raw, peer);
        Ok(())
    }

    fn get_peers(&self) -> Result<Vec<WgPeerCfg>, Box<dyn VpnctrlError>> {
        Ok(self.peers.values().cloned().collect())
    }

    fn get_peer(&self, pubkey: &String) -> Result<WgPeerCfg, Box<dyn VpnctrlError>> {
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

    fn remove_peer(&mut self, pubkey: &String) -> Result<(), Box<dyn VpnctrlError>> {
        let pk = match Key::from_base64(&pubkey) {
            Ok(x) => x,
            Err(_) => {
                return Err(Box::new(BadParameterError::new(
                    "Invalid pubkey format".to_string(),
                )))
            }
        };

        // Lookup the peer
        let mut pubkey_raw: [u8; 32] = [0; 32];
        pubkey_raw.copy_from_slice(pk.as_bytes());
        if self.peers.get(&pubkey_raw).is_none() {
            // Not exist
            return Err(Box::new(EntryNotFoundError::new(
                "Entry not found".to_string(),
            )));
        }

        // Remove peer
        match DeviceUpdate::new()
            .remove_peer_by_key(&pk)
            .apply(&self.ifname, self.backend)
        {
            Ok(_) => (),
            Err(_) => {
                return Err(Box::new(InternalError::new(
                    "Failed to update interface".to_string(),
                )));
            }
        };

        self.peers.remove(&pubkey_raw);

        Ok(())
    }

    fn get_status(&self) -> InterfaceStatus {
        self.status.clone()
    }

    fn get_trafficstats(&self) -> Result<Vec<PeerTrafficStat>, Box<dyn VpnctrlError>> {
        let dev = match Device::get(&self.ifname, self.backend) {
            Ok(x) => x,
            Err(_) => {
                return Err(Box::new(EntryNotFoundError::new(
                    "Entry not found".to_string(),
                )))
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

    fn set_ip(&mut self, ips: &[String]) -> Result<(), Box<dyn VpnctrlError>> {
        let ipns: Vec<IpNetwork> = ips
            .into_iter()
            .map(|x| x.parse())
            .filter(|x| x.is_ok())
            .map(|x| x.unwrap())
            .collect();

        for ipn in ipns {
            match netlink::set_addr(&self.ifname, ipn) {
                Ok(_) => {}
                Err(_) => {
                    return Err(Box::new(InternalError::new(
                        "Failed to set address".to_string(),
                    )))
                }
            }
        }

        Ok(())
    }

    fn add_route(&mut self, cidr: &String) -> Result<(), Box<dyn VpnctrlError>> {
        let ipn: IpNetwork = match cidr.parse() {
            Ok(x) => x,
            Err(_) => return Err(Box::new(BadParameterError::new("bad cidr".to_string()))),
        };
        match netlink::add_route(&self.ifname, self.fwmark, ipn) {
            Ok(_) => Ok(()),
            Err(_) => Err(Box::new(InternalError::new("Internal error".to_string()))),
        }
    }

    fn remove_route(&mut self, ip: &String) -> Result<(), Box<dyn VpnctrlError>> {
        Err(Box::new(InternalError::new(
            "Not implemented yet".to_string(),
        )))
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
