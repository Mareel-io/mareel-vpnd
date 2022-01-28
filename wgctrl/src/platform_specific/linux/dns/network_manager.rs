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

use std::net::IpAddr;
pub use talpid_dbus::network_manager::Error;
use talpid_dbus::network_manager::{self, DeviceConfig, NetworkManager as DBus};

pub type Result<T> = std::result::Result<T, Error>;

pub struct NetworkManager {
    pub connection: DBus,
    device: Option<String>,
    settings_backup: Option<DeviceConfig>,
}

impl NetworkManager {
    pub fn new() -> Result<Self> {
        let connection = DBus::new()?;
        connection.ensure_resolv_conf_is_managed()?;
        connection.ensure_network_manager_exists()?;
        connection.nm_version_dns_works()?;
        let manager = NetworkManager {
            connection,
            device: None,
            settings_backup: None,
        };
        Ok(manager)
    }

    pub fn set_dns(&mut self, interface_name: &str, servers: &[IpAddr]) -> Result<()> {
        let old_settings = self.connection.set_dns(interface_name, servers)?;
        self.settings_backup = Some(old_settings);
        self.device = Some(interface_name.to_string());
        Ok(())
    }

    pub fn reset(&mut self) -> Result<()> {
        if let Some(settings_backup) = self.settings_backup.take() {
            let device = match self.device.take() {
                Some(device) => device,
                None => return Ok(()),
            };
            let device_path = match self.connection.fetch_device(&device) {
                Ok(device_path) => device_path,
                Err(Error::DeviceNotFound) => return Ok(()),
                Err(error) => return Err(error),
            };

            if network_manager::device_is_ready(self.connection.get_device_state(&device_path)?) {
                self.connection
                    .reapply_settings(&device_path, settings_backup, 0u64)?;
            }
            return Ok(());
        }
        log::trace!("No DNS settings to reset");
        Ok(())
    }
}
