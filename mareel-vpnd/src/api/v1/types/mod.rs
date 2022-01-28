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

use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};

use dashmap::{DashMap, DashSet};
use prometheus::Counter;

use wgctrl::platform_specific::common::{DnsMonitor, PlatformInterface};
use wgctrl::platform_specific::Route;

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) autoalloc: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) autoalloc_v4: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) autoalloc_v6: Option<u64>,
}

pub(crate) struct IfaceState {
    pub interface: Box<dyn PlatformInterface + Send>,
    pub iface_cfg: InterfaceConfig,
    pub peer_cfgs: HashMap<String, (PeerConfig, Counter, Counter)>,
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
    pub(crate) iface_states: DashMap<String, Arc<Mutex<IfaceState>>>,
}

pub(crate) struct IpStore {
    pub(crate) v4: DashSet<u32>,
    pub(crate) v4_last_count: RwLock<u32>,
    pub(crate) v6: DashSet<u64>,
    pub(crate) v6_last_count: RwLock<u64>,
}

pub(crate) struct RouteManagerStore {
    pub route_manager: Mutex<Box<Route>>,
    pub route_store: DashMap<String, HashMap<String, bool>>,
}

pub(crate) struct DnsMonStore {
    pub dnsmon: Arc<Mutex<DnsMonitor>>,
}
