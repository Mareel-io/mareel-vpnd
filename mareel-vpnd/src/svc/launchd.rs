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

const LAUNCHD_UNIT_PATH: &str = "/Library/LaunchDaemons/io.mareel.vpnd.plist";

pub fn install(config: &Option<String>) -> Result<(), ()> {
    let service_binary_path = ::std::env::current_exe().unwrap();
    let mut working_dir = ::std::env::current_exe().unwrap();
    working_dir.pop();
    // ugly xml...
    let exec_cmd = match config {
        Some(x) => format!(
            "<string>{}</string><string>--config</string><string>{}</string>",
            service_binary_path.to_string_lossy(),
            std::borrow::Cow::Borrowed(x),
        ),
        None => format!("<string>{}</string>", service_binary_path.to_string_lossy()),
    };
    let launchd_unit_path: PathBuf = LAUNCHD_UNIT_PATH.into();
    let launchd_unit = format!(
        r##"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
        <key>StandardErrorPath</key>
        <string>/var/log/mareel-vpnd.log</string>
        <key>StandardOutPath</key>
        <string>/var/log/mareel-vpnd.log</string>
        <key>UserName</key>
        <string>root</string>
        <key>KeepAlive</key>
        <true/>
        <key>Label</key>
        <string>io.mareel.vpnd</string>
        <key>RunAtLoad</key>
        <true/>
        <key>SoftResourceLimits</key>
        <dict>
                <key>NumberOfFiles</key>
                <integer>1024</integer>
        </dict>
        <key>WorkingDirectory</key>
        <string>{}</string>
        <key>ProgramArguments</key>
        <array>
            {}
        </array>
</dict>
</plist>
"##,
        working_dir.to_str().unwrap(),
        exec_cmd
    );

    println!(
        "Installed unit with workdir {}",
        working_dir.to_str().unwrap()
    );

    let mut unit_file = File::create(launchd_unit_path).unwrap();

    unit_file.write_all(launchd_unit.as_bytes()).unwrap();
    unit_file.sync_all().unwrap();
    drop(unit_file);

    Ok(())
}

pub fn start() -> Result<(), ()> {
    Command::new("launchctl")
        .arg("load")
        .arg(LAUNCHD_UNIT_PATH)
        .output()
        .expect("Failed to start service!");
    Ok(())
}

pub fn stop() -> Result<(), ()> {
    Command::new("launchctl")
        .arg("unload")
        .arg(LAUNCHD_UNIT_PATH)
        .output()
        .expect("Failed to stop service!");
    Ok(())
}

pub fn uninstall() -> Result<(), ()> {
    Command::new("launchctl")
        .arg("unload")
        .arg(LAUNCHD_UNIT_PATH)
        .output()
        .expect("Failed to stop service!");

    remove_file(LAUNCHD_UNIT_PATH).expect("Failed to remove service!");
    Ok(())
}
