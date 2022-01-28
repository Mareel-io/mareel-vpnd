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

use libc::{c_char, c_void};
use std::{ffi::CStr, io, ptr};
use winapi::um::{stringapiset::MultiByteToWideChar, winnls::CP_ACP};

/// Logging callback type.
pub type LogSink = extern "system" fn(level: log::Level, msg: *const c_char, context: *mut c_void);

/// Logging callback implementation.
pub extern "system" fn log_sink(level: log::Level, msg: *const c_char, context: *mut c_void) {
    if msg.is_null() {
        log::error!("Log message from FFI boundary is NULL");
    } else {
        let rust_log_level = log::Level::from(level);
        let target = if context.is_null() {
            "UNKNOWN".into()
        } else {
            unsafe { CStr::from_ptr(context as *const _).to_string_lossy() }
        };

        let mb_string = unsafe { CStr::from_ptr(msg) };

        let managed_msg = match multibyte_to_wide(mb_string, CP_ACP) {
            Ok(wide_str) => String::from_utf16_lossy(&wide_str),
            // Best effort:
            Err(_) => mb_string.to_string_lossy().into_owned(),
        };

        log::logger().log(
            &log::Record::builder()
                .level(rust_log_level)
                .target(&target)
                .args(format_args!("{}", managed_msg))
                .build(),
        );
    }
}

fn multibyte_to_wide(mb_string: &CStr, codepage: u32) -> Result<Vec<u16>, io::Error> {
    if unsafe { *mb_string.as_ptr() } == 0 {
        return Ok(vec![]);
    }

    let wc_size =
        unsafe { MultiByteToWideChar(codepage, 0, mb_string.as_ptr(), -1, ptr::null_mut(), 0) };

    if wc_size == 0 {
        return Err(io::Error::last_os_error());
    }

    let mut wc_buffer = Vec::with_capacity(wc_size as usize);

    let chars_written = unsafe {
        MultiByteToWideChar(
            codepage,
            0,
            mb_string.as_ptr(),
            -1,
            wc_buffer.as_mut_ptr(),
            wc_size,
        )
    };

    if chars_written == 0 {
        return Err(io::Error::last_os_error());
    }

    unsafe { wc_buffer.set_len((chars_written - 1) as usize) };

    Ok(wc_buffer)
}
