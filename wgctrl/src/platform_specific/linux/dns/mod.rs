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

pub(self) mod iface;

#[cfg(feature = "dbus")]
mod network_manager;
mod resolvconf;
mod static_resolv_conf;
#[cfg(feature = "dbus")]
pub(self) mod systemd_resolved;

#[cfg(feature = "dbus")]
use self::{network_manager::NetworkManager, systemd_resolved::SystemdResolved};

use self::{resolvconf::Resolvconf, static_resolv_conf::StaticResolvConf};

use std::{env, fmt, net::IpAddr};

const RESOLV_CONF_PATH: &str = "/etc/resolv.conf";

pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can happen in the Linux DNS monitor
#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// Error in systemd-resolved DNS monitor
    #[cfg(feature = "dbus")]
    #[error(display = "Error in systemd-resolved DNS monitor")]
    SystemdResolved(#[error(source)] systemd_resolved::Error),

    /// Error in NetworkManager DNS monitor
    #[cfg(feature = "dbus")]
    #[error(display = "Error in NetworkManager DNS monitor")]
    NetworkManager(#[error(source)] network_manager::Error),

    /// Error in resolvconf DNS monitor
    #[error(display = "Error in resolvconf DNS monitor")]
    Resolvconf(#[error(source)] resolvconf::Error),

    /// Error in static /etc/resolv.conf DNS monitor
    #[error(display = "Error in static /etc/resolv.conf DNS monitor")]
    StaticResolvConf(#[error(source)] static_resolv_conf::Error),

    /// No suitable DNS monitor implementation detected
    #[error(display = "No suitable DNS monitor implementation detected")]
    NoDnsMonitor,
}

pub struct DnsMonitor {
    handle: tokio::runtime::Handle,
    inner: Option<DnsMonitorHolder>,
}

impl super::super::common::DnsMonitorT for DnsMonitor {
    type Error = Error;

    fn new(handle: tokio::runtime::Handle) -> Result<Self> {
        Ok(DnsMonitor {
            handle,
            inner: None,
        })
    }

    fn set(&mut self, interface: &str, servers: &[IpAddr]) -> Result<()> {
        self.reset()?;
        // Creating a new DNS monitor for each set, in case the system changed how it manages DNS.
        let mut inner = DnsMonitorHolder::new()?;
        if !servers.is_empty() {
            inner.set(&self.handle, interface, servers)?;
            self.inner = Some(inner);
        }
        Ok(())
    }

    fn reset(&mut self) -> Result<()> {
        if let Some(mut inner) = self.inner.take() {
            inner.reset(&self.handle)?;
        }
        Ok(())
    }
}

pub enum DnsMonitorHolder {
    #[cfg(feature = "dbus")]
    SystemdResolved(SystemdResolved),
    #[cfg(feature = "dbus")]
    NetworkManager(NetworkManager),
    Resolvconf(Resolvconf),
    StaticResolvConf(StaticResolvConf),
}

impl fmt::Display for DnsMonitorHolder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::DnsMonitorHolder::*;
        let name = match self {
            Resolvconf(..) => "resolvconf",
            StaticResolvConf(..) => "/etc/resolv.conf",
            #[cfg(feature = "dbus")]
            SystemdResolved(..) => "systemd-resolved",
            #[cfg(feature = "dbus")]
            NetworkManager(..) => "network manager",
        };
        f.write_str(name)
    }
}

impl DnsMonitorHolder {
    fn new() -> Result<Self> {
        let dns_module = env::var_os("TALPID_DNS_MODULE");

        let manager = match dns_module.as_ref().and_then(|value| value.to_str()) {
            Some("static-file") => DnsMonitorHolder::StaticResolvConf(StaticResolvConf::new()?),
            Some("resolvconf") => DnsMonitorHolder::Resolvconf(Resolvconf::new()?),
            #[cfg(feature = "dbus")]
            Some("systemd") => DnsMonitorHolder::SystemdResolved(SystemdResolved::new()?),
            #[cfg(feature = "dbus")]
            Some("network-manager") => DnsMonitorHolder::NetworkManager(NetworkManager::new()?),
            Some(_) | None => Self::with_detected_dns_manager()?,
        };
        log::debug!("Managing DNS via {}", manager);
        Ok(manager)
    }

    fn with_detected_dns_manager() -> Result<Self> {
        #[cfg(not(feature = "dbus"))]
        return Resolvconf::new()
            .map(DnsMonitorHolder::Resolvconf)
            .or_else(|_| StaticResolvConf::new().map(DnsMonitorHolder::StaticResolvConf))
            .map_err(|_| Error::NoDnsMonitor);

        #[cfg(feature = "dbus")]
        SystemdResolved::new()
            .map(DnsMonitorHolder::SystemdResolved)
            .or_else(|err| {
                match err {
                    systemd_resolved::Error::SystemdResolvedError(
                        systemd_resolved::SystemdDbusError::NoSystemdResolved(_),
                    ) => (),
                    other_error => {
                        log::debug!("NetworkManager is not being used because {}", other_error)
                    }
                }
                NetworkManager::new().map(DnsMonitorHolder::NetworkManager)
            })
            .or_else(|_| Resolvconf::new().map(DnsMonitorHolder::Resolvconf))
            .or_else(|_| StaticResolvConf::new().map(DnsMonitorHolder::StaticResolvConf))
            .map_err(|_| Error::NoDnsMonitor)
    }

    fn set(
        &mut self,
        handle: &tokio::runtime::Handle,
        interface: &str,
        servers: &[IpAddr],
    ) -> Result<()> {
        use self::DnsMonitorHolder::*;
        match self {
            Resolvconf(ref mut resolvconf) => resolvconf.set_dns(interface, servers)?,
            StaticResolvConf(ref mut static_resolv_conf) => {
                static_resolv_conf.set_dns(servers.to_vec())?
            }
            #[cfg(feature = "dbus")]
            SystemdResolved(ref mut systemd_resolved) => {
                handle.block_on(systemd_resolved.set_dns(interface, &servers))?
            }
            #[cfg(feature = "dbus")]
            NetworkManager(ref mut network_manager) => {
                network_manager.set_dns(interface, servers)?
            }
        }
        Ok(())
    }

    fn reset(&mut self, handle: &tokio::runtime::Handle) -> Result<()> {
        use self::DnsMonitorHolder::*;
        match self {
            Resolvconf(ref mut resolvconf) => resolvconf.reset()?,
            StaticResolvConf(ref mut static_resolv_conf) => static_resolv_conf.reset()?,
            #[cfg(feature = "dbus")]
            SystemdResolved(ref mut systemd_resolved) => {
                handle.block_on(systemd_resolved.reset())?
            }
            #[cfg(feature = "dbus")]
            NetworkManager(ref mut network_manager) => network_manager.reset()?,
        }
        Ok(())
    }
}

/// Returns true if DnsMonitor will use NetworkManager to manage DNS.
#[cfg(feature = "dbus")]
pub fn will_use_nm() -> bool {
    SystemdResolved::new().is_err() && NetworkManager::new().is_ok()
}

#[cfg(not(feature = "dbus"))]
pub fn will_use_nm() -> bool {
    false
}
