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

use super::iface::{iface_index, IfaceIndexLookupError};
use std::net::IpAddr;
use talpid_dbus::systemd_resolved::{AsyncHandle, SystemdResolved as DbusInterface};
use talpid_types::ErrorExt;

pub(crate) use talpid_dbus::systemd_resolved::Error as SystemdDbusError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "systemd-resolved operation failed")]
    SystemdResolvedError(#[error(source)] SystemdDbusError),

    #[error(display = "Failed to resolve interface index with error {}", _0)]
    InterfaceNameError(#[error(source)] IfaceIndexLookupError),
}

pub struct SystemdResolved {
    pub dbus_interface: AsyncHandle,
    tunnel_index: u32,
}

impl SystemdResolved {
    pub fn new() -> Result<Self> {
        let dbus_interface = DbusInterface::new()?.async_handle();

        let systemd_resolved = SystemdResolved {
            dbus_interface,
            tunnel_index: 0,
        };

        Ok(systemd_resolved)
    }

    pub async fn set_dns(
        &mut self,
        interface_name: &str,
        servers: &[IpAddr],
    ) -> Result<()> {
        let tunnel_index = iface_index(interface_name)?;
        self.tunnel_index = tunnel_index;

        if let Err(error) = self.dbus_interface.disable_dot(self.tunnel_index).await {
            log::error!("Failed to disable DoT: {}", error.display_chain());
        }

        if let Err(error) = self
            .dbus_interface
            .set_domains(tunnel_index, &[(".", true)])
            .await
        {
            log::error!("Failed to set search domains: {}", error.display_chain());
        }

        let _ = self
            .dbus_interface
            .set_dns(self.tunnel_index, servers.to_vec())
            .await?;

        Ok(())
    }

    pub async fn reset(&mut self) -> Result<()> {
        if let Err(error) = self
            .dbus_interface
            .set_domains(self.tunnel_index, &[])
            .await
        {
            log::error!("Failed to set search domains: {}", error.display_chain());
        }

        let _ = self
            .dbus_interface
            .set_dns(self.tunnel_index, vec![])
            .await?;

        Ok(())
    }
}
