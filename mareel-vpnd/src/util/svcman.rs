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

use super::super::svc;

pub(crate) fn svc_install(method: &str, config: &Option<String>) -> Result<(), ()> {
    #[cfg(target_os = "linux")]
    {
        match method {
            "systemd" => svc::systemd::install(config).unwrap(),
            _ => panic!("Not supported feature: {}", method),
        };
        return Ok(());
    }
    #[cfg(target_os = "windows")]
    {
        match method {
            "winsvc" => svc::winsvc::install(config).unwrap(),
            _ => panic!("Not supported feature: {}", method),
        };
        return Ok(());
    }
    #[cfg(target_os = "macos")]
    {
        match method {
            "launchd" => svc::launchd::install(config).unwrap(),
            _ => panic!("Not supported feature: {}", method),
        };
        return Ok(());
    }
    #[allow(unreachable_code)]
    {
        panic!("Not supported yet!");
    }
}

pub(crate) fn svc_uninstall(method: &str) -> Result<(), ()> {
    #[cfg(target_os = "linux")]
    {
        match method {
            "systemd" => svc::systemd::uninstall().unwrap(),
            _ => panic!("Not supported feature: {}", method),
        };
        return Ok(());
    }
    #[cfg(target_os = "windows")]
    {
        match method {
            "winsvc" => svc::winsvc::uninstall().unwrap(),
            _ => panic!("Not supported feature: {}", method),
        };
        return Ok(());
    }
    #[cfg(target_os = "macos")]
    {
        match method {
            "launchd" => svc::launchd::uninstall().unwrap(),
            _ => panic!("Not supported feature: {}", method),
        };
        return Ok(());
    }
    #[allow(unreachable_code)]
    {
        panic!("Not supported yet!");
    }
}

pub(crate) fn svc_start(method: &str) -> Result<(), ()> {
    #[cfg(target_os = "linux")]
    {
        match method {
            "systemd" => svc::systemd::start().unwrap(),
            _ => panic!("Not supported feature: {}", method),
        };
        return Ok(());
    }
    #[cfg(target_os = "windows")]
    {
        match method {
            "winsvc" => svc::winsvc::start().unwrap(),
            _ => panic!("Not supported feature: {}", method),
        };
        return Ok(());
    }
    #[cfg(target_os = "macos")]
    {
        match method {
            "launchd" => svc::launchd::start().unwrap(),
            _ => panic!("Not supported feature: {}", method),
        };
        return Ok(());
    }
    #[allow(unreachable_code)]
    {
        panic!("Not supported yet!");
    }
}

pub(crate) fn svc_stop(method: &str) -> Result<(), ()> {
    #[cfg(target_os = "linux")]
    {
        match method {
            "systemd" => svc::systemd::stop().unwrap(),
            _ => panic!("Not supported feature: {}", method),
        };
        return Ok(());
    }
    #[cfg(target_os = "windows")]
    {
        match method {
            "winsvc" => svc::winsvc::stop().unwrap(),
            _ => panic!("Not supported feature: {}", method),
        };
        return Ok(());
    }
    #[cfg(target_os = "macos")]
    {
        match method {
            "launchd" => svc::launchd::stop().unwrap(),
            _ => panic!("Not supported feature: {}", method),
        };
        return Ok(());
    }
    #[allow(unreachable_code)]
    {
        panic!("Not supported yet!");
    }
}
