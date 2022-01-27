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

use super::super::common::PlatformRoute;
use crate::vpnctrl::error::VpnctrlError;

pub struct Route {}

impl PlatformRoute for Route {
    fn new(_fwmark: u32) -> Result<Self, VpnctrlError>
    where
        Self: Sized,
    {
        Ok(Self {})
    }

    fn init(&mut self) -> Result<(), VpnctrlError> {
        Ok(())
    }

    fn add_route(&mut self, _ifname: &str, _ip: &str) -> Result<(), VpnctrlError> {
        Ok(()) // wireguard-nt library does some routing stuff, so just ignore it for now...
    }

    fn remove_route(&mut self, _ifname: &str, _ip: &str) -> Result<(), VpnctrlError> {
        Err(VpnctrlError::Internal {
            msg: "Not implemented yet".to_string(),
        })
    }

    fn add_route_bypass(&mut self, _address: &str) -> Result<(), VpnctrlError> {
        Ok(())
    }

    fn remove_route_bypass(&mut self, _address: &str) -> Result<(), VpnctrlError> {
        Ok(())
    }

    fn get_route_bypass(&self) -> Result<Vec<String>, VpnctrlError> {
        Ok(vec![])
    }

    fn backup_default_route(&mut self) -> Result<(), VpnctrlError> {
        Ok(())
    }

    fn remove_default_route(&mut self) -> Result<(), VpnctrlError> {
        Ok(())
    }

    fn restore_default_route(&mut self) -> Result<(), VpnctrlError> {
        Ok(())
    }
}
