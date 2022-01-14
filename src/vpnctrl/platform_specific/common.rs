use std::fmt;

use crate::vpnctrl::error::VpnctrlError;

#[derive(Debug, Clone)]
pub struct PlatformError {
    msg: String,
}

impl fmt::Display for PlatformError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PlatformError: {}", self.msg)
    }
}

impl VpnctrlError for PlatformError {}

impl PlatformError {
    pub fn new(msg: String) -> PlatformError {
        PlatformError { msg }
    }
}

#[derive(Clone)]
pub struct WgIfCfg {
    pub listen_port: Option<u16>,
    pub privkey: String,
    pub fwmark: u32,
}

#[derive(Clone)]
pub struct WgPeerCfg {
    pub pubkey: String,
    pub psk: Option<String>,
    pub endpoint: Option<String>,
    pub allowed_ips: Vec<String>,
    pub keep_alive: Option<u16>,
}

#[derive(Clone)]
pub enum InterfaceStatus {
    Stopped,
    Running,
}

impl ToString for InterfaceStatus {
    fn to_string(&self) -> String {
        match self {
            InterfaceStatus::Stopped => "stopped".to_string(),
            InterfaceStatus::Running => "running".to_string(),
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(crate = "rocket::serde")]
pub struct PeerTrafficStat {
    pub(crate) pubkey: String,
    pub(crate) rx_bytes: u64,
    pub(crate) tx_bytes: u64,
}

pub trait PlatformInterface {
    fn new(name: &str) -> Result<Self, PlatformError>
    where
        Self: Sized;
    fn set_config(&mut self, cfg: WgIfCfg) -> Result<(), Box<dyn VpnctrlError>>;
    fn add_peer(&mut self, peer: WgPeerCfg) -> Result<(), Box<dyn VpnctrlError>>;
    fn get_peers(&self) -> Result<Vec<WgPeerCfg>, Box<dyn VpnctrlError>>;
    fn get_peer(&self, pubkey: &str) -> Result<WgPeerCfg, Box<dyn VpnctrlError>>;
    fn remove_peer(&mut self, pubkey: &str) -> Result<(), Box<dyn VpnctrlError>>;
    fn get_status(&self) -> InterfaceStatus;
    fn get_trafficstats(&self) -> Result<Vec<PeerTrafficStat>, Box<dyn VpnctrlError>>;
    fn up(&mut self) -> bool;
    fn down(&mut self) -> bool;
    fn set_ip(&mut self, ips: &[String]) -> Result<(), Box<dyn VpnctrlError>>;
}

pub trait PlatformRoute {
    fn new(fwmark: u32) -> Result<Self, PlatformError>
    where
        Self: Sized;
    fn init(&mut self) -> Result<(), Box<dyn VpnctrlError>>;
    fn add_route(&mut self, ifname: &str, cidr: &str) -> Result<(), Box<dyn VpnctrlError>>;
    fn remove_route(&mut self, ifname: &str, cidr: &str)
        -> Result<(), Box<dyn VpnctrlError>>;
    fn add_route_bypass(&mut self, address: &str) -> Result<(), Box<dyn VpnctrlError>>;
    fn backup_default_route(&mut self) -> Result<(), Box<dyn VpnctrlError>>;
    fn remove_default_route(&mut self) -> Result<(), Box<dyn VpnctrlError>>;
    fn restore_default_route(&mut self) -> Result<(), Box<dyn VpnctrlError>>;
}
