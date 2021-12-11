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

pub trait PlatformInterface {
    fn new(name: &str) -> Result<Self, PlatformError>
    where
        Self: Sized;
    fn set_config(&mut self, cfg: WgIfCfg) -> Result<(), Box<dyn VpnctrlError>>;
    fn add_peer(&mut self, peer: WgPeerCfg) -> Result<(), Box<dyn VpnctrlError>>;
    fn get_peers(&self) -> Result<Vec<WgPeerCfg>, Box<dyn VpnctrlError>>;
    fn get_peer(&self, pubkey: &String) -> Result<WgPeerCfg, Box<dyn VpnctrlError>>;
    fn remove_peer(&mut self, pubkey: &String) -> Result<(), Box<dyn VpnctrlError>>;
    fn get_status(&self) -> InterfaceStatus;
    fn up(&mut self) -> bool;
    fn down(&mut self) -> bool;
}
