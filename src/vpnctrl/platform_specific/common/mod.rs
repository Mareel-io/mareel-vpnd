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

pub struct WgIfCfg {
    pub listen_port: Option<u16>,
    pub privkey: String,
}

pub struct WgPeerCfg {
    pub pubkey: String,
}

pub trait PlatformInterface {
    fn new(name: &str) -> Result<Self, PlatformError>
    where
        Self: Sized;
    fn set_config(&mut self, cfg: WgIfCfg) -> Result<(), Box<dyn VpnctrlError>>;
    fn add_peer(&mut self, peer: WgPeerCfg) -> Result<(), Box<dyn VpnctrlError>>;
    fn remove_peer(&mut self, pubkey: String) -> Result<(), Box<dyn VpnctrlError>>;
    fn up(&self) -> bool;
    fn down(&self) -> bool;
}