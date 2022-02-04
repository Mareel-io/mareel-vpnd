/*
 * SPDX-FileCopyrightText: 2022 Empo Inc.
 * SPDX-FileCopyrightText: 2022 Mullvad VPN AB
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

use custom_error::custom_error;

use crate::error::VpnctrlError;

custom_error! {pub PlatformError
    VpnctrlError{source: VpnctrlError} = "VpnctrlError",
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
    pub pubkey: String,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
}

pub trait PlatformInterface {
    fn new(name: &str) -> Result<Self, VpnctrlError>
    where
        Self: Sized;
    fn set_config(&mut self, cfg: WgIfCfg) -> Result<(), VpnctrlError>;
    fn add_peer(&mut self, peer: WgPeerCfg) -> Result<(), VpnctrlError>;
    fn get_peers(&self) -> Result<Vec<WgPeerCfg>, VpnctrlError>;
    fn get_peer(&self, pubkey: &str) -> Result<WgPeerCfg, VpnctrlError>;
    fn remove_peer(&mut self, pubkey: &str) -> Result<(), VpnctrlError>;
    fn get_status(&self) -> InterfaceStatus;
    fn get_trafficstats(&self) -> Result<Vec<PeerTrafficStat>, VpnctrlError>;
    fn get_platformid(&self) -> Result<String, VpnctrlError>;
    fn up(&mut self) -> bool;
    fn down(&mut self) -> bool;
    fn set_ip(&mut self, ips: &[String]) -> Result<(), VpnctrlError>;
}

pub trait PlatformRoute {
    fn new(fwmark: u32) -> Result<Self, VpnctrlError>
    where
        Self: Sized;
    fn init(&mut self) -> Result<(), VpnctrlError>;
    fn add_route(&mut self, ifname: &str, cidr: &str) -> Result<(), VpnctrlError>;
    fn remove_route(&mut self, ifname: &str, cidr: &str) -> Result<(), VpnctrlError>;
    fn add_route_bypass(&mut self, address: &str) -> Result<(), VpnctrlError>;
    fn remove_route_bypass(&mut self, address: &str) -> Result<(), VpnctrlError>;
    fn get_route_bypass(&self) -> Result<Vec<String>, VpnctrlError>;
    fn backup_default_route(&mut self) -> Result<(), VpnctrlError>;
    fn remove_default_route(&mut self) -> Result<(), VpnctrlError>;
    fn restore_default_route(&mut self) -> Result<(), VpnctrlError>;
}

// Imported from Mullvad talpid-core
use std::net::IpAddr;

#[cfg(target_os = "linux")]
use super::super::platform_specific::linux::dns;

#[cfg(target_os = "macos")]
use super::super::platform_specific::macos::dns;

#[cfg(windows)]
use super::super::platform_specific::windows::dns;

pub use dns::Error;

/// Sets and monitors system DNS settings. Makes sure the desired DNS servers are being used.
pub struct DnsMonitor {
    inner: dns::DnsMonitor,
}

impl DnsMonitor {
    /// Returns a new `DnsMonitor` that can set and monitor the system DNS.
    pub fn new(handle: tokio::runtime::Handle) -> Result<Self, Error> {
        Ok(DnsMonitor {
            inner: dns::DnsMonitor::new(handle)?,
        })
    }

    /// Returns a map of interfaces and respective list of resolvers that don't contain our
    /// changes.
    #[cfg(target_os = "macos")]
    pub fn get_system_config(&self) -> Result<Option<(String, Vec<IpAddr>)>, Error> {
        self.inner.get_system_config()
    }

    /// Set DNS to the given servers. And start monitoring the system for changes.
    pub fn set(&mut self, interface: &str, servers: &[IpAddr]) -> Result<(), Error> {
        log::info!(
            "Setting DNS servers to {}",
            servers
                .iter()
                .map(|ip| ip.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        );
        self.inner.set(interface, servers)
    }

    /// Reset system DNS settings to what it was before being set by this instance.
    /// This succeeds if the interface does not exist.
    pub fn reset(&mut self) -> Result<(), Error> {
        log::info!("Resetting DNS");
        self.inner.reset()
    }
}

pub trait DnsMonitorT: Sized {
    type Error: std::error::Error;

    fn new(handle: tokio::runtime::Handle) -> Result<Self, Self::Error>;

    fn set(&mut self, interface: &str, servers: &[IpAddr]) -> Result<(), Self::Error>;

    fn reset(&mut self) -> Result<(), Self::Error>;
}
