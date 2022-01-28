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

#![cfg(target_os = "linux")]
//! DBus system connection
pub use dbus;
use dbus::blocking::SyncConnection;
use std::sync::{Arc, Mutex};
pub mod network_manager;
pub mod systemd_resolved;

lazy_static::lazy_static! {
    static ref DBUS_CONNECTION: Mutex<Option<Arc<SyncConnection>>> = Mutex::new(None);
}

/// Reuse or create a system DBus connection.
pub fn get_connection() -> Result<Arc<SyncConnection>, dbus::Error> {
    let mut connection = DBUS_CONNECTION.lock().expect("DBus lock poisoned");
    match &*connection {
        Some(existing_connection) => Ok(existing_connection.clone()),
        None => {
            let new_connection = Arc::new(SyncConnection::new_system()?);
            *connection = Some(new_connection.clone());
            Ok(new_connection)
        }
    }
}
