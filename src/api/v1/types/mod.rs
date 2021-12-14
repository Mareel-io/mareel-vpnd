use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::vpnctrl::platform_specific::common::PlatformInterface;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(crate = "rocket::serde")]
pub(crate) struct InterfaceConfig {
    pub(crate) name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) private_key: Option<String>,
    #[serde(skip_deserializing)]
    pub(crate) public_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) listen_port: Option<u16>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(crate = "rocket::serde")]
pub(crate) struct PeerConfig {
    pub(crate) pubkey: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) psk: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) endpoint: Option<String>,
    pub(crate) allowed_ips: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) keepalive: Option<u16>,
}

pub(crate) struct IfaceState {
    pub interface: Box<dyn PlatformInterface + Send>,
    pub iface_cfg: InterfaceConfig,
    pub peer_cfgs: HashMap<String, PeerConfig>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(crate = "rocket::serde")]
pub(crate) struct DaemonControlMessage {
    pub(crate) magic: u32,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(crate = "rocket::serde")]
pub(crate) struct IpConfigurationMessage {
    pub(crate) ipaddr: Vec<String>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(crate = "rocket::serde")]
pub(crate) struct RouteConfigurationMessage {
    pub(crate) cidr: String,
}

pub(crate) struct InterfaceStore {
    pub(crate) iface_states: Mutex<HashMap<String, Arc<Mutex<IfaceState>>>>,
}
