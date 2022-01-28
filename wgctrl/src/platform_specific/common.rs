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
