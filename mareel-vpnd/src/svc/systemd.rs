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

use std::fs::{remove_file, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

use shell_escape::unix::escape;

pub fn install(config: &Option<String>) -> Result<(), ()> {
    let service_binary_path = ::std::env::current_exe().unwrap();

    let exec_cmd = match config {
        Some(x) => format!(
            "{} --config {}",
            escape(service_binary_path.to_string_lossy()),
            escape(std::borrow::Cow::Borrowed(x)),
        ),
        None => format!("{}", escape(service_binary_path.to_string_lossy())),
    };
    // Installation path
    let systemd_unit_path: PathBuf = "/etc/systemd/system/mareel-vpnd.service".into();
    let systemd_unit = format!(
        r##"
# Systemd service unit file for the Mareel VPN daemon

[Unit]
Description=Mareel VPN daemon
Wants=network.target
After=network-online.target
After=NetworkManager.service
After=systemd-resolved.service
StartLimitBurst=5
StartLimitIntervalSec=20

[Service]
Restart=always
RestartSec=1
ExecStart={}

[Install]
WantedBy=multi-user.target
"##,
        exec_cmd
    );

    let mut unit_file = File::create(systemd_unit_path).unwrap();

    unit_file.write_all(systemd_unit.as_bytes()).unwrap();
    unit_file.sync_all().unwrap();
    drop(unit_file);

    // TODO: Change config file mode

    Command::new("systemctl")
        .arg("daemon-reload")
        .output()
        .expect("Failed to reload daemon!");

    Command::new("systemctl")
        .arg("enable")
        .arg("mareel-vpnd.service")
        .output()
        .expect("Failed to enable service!");

    Ok(())
}

pub fn start() -> Result<(), ()> {
    Command::new("systemctl")
        .arg("start")
        .arg("mareel-vpnd.service")
        .output()
        .expect("Failed to start service!");
    Ok(())
}

pub fn stop() -> Result<(), ()> {
    Command::new("systemctl")
        .arg("stop")
        .arg("mareel-vpnd.service")
        .output()
        .expect("Failed to stop service!");
    Ok(())
}

pub fn uninstall() -> Result<(), ()> {
    Command::new("systemctl")
        .arg("disable")
        .arg("--now")
        .arg("mareel-vpnd.service")
        .output()
        .expect("Failed to disable service!");

    let systemd_unit_path: PathBuf = "/etc/systemd/system/mareel-vpnd.service".into();
    remove_file(systemd_unit_path).expect("Failed to remove service!");

    Command::new("systemctl")
        .arg("daemon-reload")
        .output()
        .expect("Failed to reload daemon!");

    Ok(())
}
