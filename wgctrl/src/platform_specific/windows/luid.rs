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

use std::{
    ffi::{OsStr, OsString},
    io,
    os::windows::{
        ffi::{OsStrExt, OsStringExt},
    },
    time::{Duration},
};
use winapi::shared::{
    ifdef::NET_LUID,
    netioapi::{ConvertInterfaceAliasToLuid, ConvertInterfaceLuidToAlias},
    ntddndis::NDIS_IF_MAX_STRING_SIZE,
    winerror::{NO_ERROR},
    nldef::NL_DAD_STATE,
    ws2def::{
        AF_INET, AF_INET6},
};

/// Result type for this module.
pub type Result<T> = std::result::Result<T, Error>;

const DAD_CHECK_TIMEOUT: Duration = Duration::from_secs(5);
const DAD_CHECK_INTERVAL: Duration = Duration::from_millis(100);

/// Errors returned by some functions in this module.
#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    /// Error returned from `ConvertInterfaceAliasToLuid`
    #[cfg(windows)]
    #[error(display = "Cannot find LUID for virtual adapter")]
    NoDeviceLuid(#[error(source)] io::Error),

    /// Error returned from `GetUnicastIpAddressTable`/`GetUnicastIpAddressEntry`
    #[cfg(windows)]
    #[error(display = "Failed to obtain unicast IP address table")]
    ObtainUnicastAddress(#[error(source)] io::Error),

    /// `GetUnicastIpAddressTable` contained no addresses for the interface
    #[cfg(windows)]
    #[error(display = "Found no addresses for the given adapter")]
    NoUnicastAddress,

    /// Unexpected DAD state returned for a unicast address
    #[cfg(windows)]
    #[error(display = "Unexpected DAD state")]
    DadStateError(#[error(source)] DadStateError),

    /// DAD check failed.
    #[cfg(windows)]
    #[error(display = "Timed out waiting on tunnel device")]
    DeviceReadyTimeout,

    /// Unicast DAD check fail.
    #[cfg(windows)]
    #[error(display = "Unicast channel sender was unexpectedly dropped")]
    UnicastSenderDropped,

    /// Unknown address family
    #[error(display = "Unknown address family: {}", _0)]
    UnknownAddressFamily(i32),
}

/// Handles cases where there DAD state is neither tentative nor preferred.
#[cfg(windows)]
#[derive(err_derive::Error, Debug)]
pub enum DadStateError {
    /// Invalid DAD state.
    #[error(display = "Invalid DAD state")]
    Invalid,

    /// Duplicate unicast address.
    #[error(display = "A duplicate IP address was detected")]
    Duplicate,

    /// Deprecated unicast address.
    #[error(display = "The IP address has been deprecated")]
    Deprecated,

    /// Unknown DAD state constant.
    #[error(display = "Unknown DAD state: {}", _0)]
    Unknown(u32),
}

#[cfg(windows)]
#[allow(non_upper_case_globals)]
impl From<NL_DAD_STATE> for DadStateError {
    fn from(state: NL_DAD_STATE) -> DadStateError {
        use winapi::shared::nldef::*;
        match state {
            IpDadStateInvalid => DadStateError::Invalid,
            IpDadStateDuplicate => DadStateError::Duplicate,
            IpDadStateDeprecated => DadStateError::Deprecated,
            other => DadStateError::Unknown(other),
        }
    }
}

/// Address family. These correspond to the `AF_*` constants.
#[derive(Debug, Clone, Copy)]
pub enum AddressFamily {
    /// IPv4 address family
    Ipv4 = AF_INET as isize,
    /// IPv6 address family
    Ipv6 = AF_INET6 as isize,
}


/// Returns the LUID of an interface given its alias.
pub fn luid_from_alias<T: AsRef<OsStr>>(alias: T) -> io::Result<NET_LUID> {
    let alias_wide: Vec<u16> = alias
        .as_ref()
        .encode_wide()
        .chain(std::iter::once(0u16))
        .collect();
    let mut luid: NET_LUID = unsafe { std::mem::zeroed() };
    let status = unsafe { ConvertInterfaceAliasToLuid(alias_wide.as_ptr(), &mut luid) };
    if status != NO_ERROR {
        return Err(io::Error::from_raw_os_error(status as i32));
    }
    Ok(luid)
}

/// Returns the alias of an interface given its LUID.
pub fn alias_from_luid(luid: &NET_LUID) -> io::Result<OsString> {
    let mut buffer = [0u16; NDIS_IF_MAX_STRING_SIZE + 1];
    let status =
        unsafe { ConvertInterfaceLuidToAlias(luid, &mut buffer[0] as *mut _, buffer.len()) };
    if status != NO_ERROR {
        return Err(io::Error::from_raw_os_error(status as i32));
    }
    let nul = buffer.iter().position(|&c| c == 0u16).unwrap();
    Ok(OsString::from_wide(&buffer[0..nul]))
}
