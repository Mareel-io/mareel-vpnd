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

use std::{ffi::OsStr, fs, os::unix::ffi::OsStrExt, path::PathBuf};

pub const SPLIT_TUNNEL_CGROUP_NAME: &str = "mullvad-exclusions";

/// Find the path of the cgroup v1 net_cls controller mount if it exists
pub fn find_net_cls_mount() -> std::io::Result<Option<PathBuf>> {
    let mounts = fs::read("/proc/mounts")?;
    Ok(find_net_cls_mount_inner(&mounts))
}

fn find_net_cls_mount_inner(mounts: &[u8]) -> Option<PathBuf> {
    mounts
        .split(|byte| *byte == b'\n')
        .find_map(parse_mount_line)
}

fn parse_mount_line(line: &[u8]) -> Option<PathBuf> {
    // Each line contains multiple values seperated by space.
    // `cgroup /sys/fs/cgroup/net_cls,net_prio cgroup
    // rw,nosuid,nodev,noexec,relatime,net_cls,net_prio 0 0`  Value meanings:
    // 1. device type
    // 2. mount path
    // 3. filesystem type
    // 4. mount options
    // 5./6. legacy dummy values
    let mut parts = line.split(|byte| *byte == b' ');
    let _device_type = parts.next()?;
    let mount_path = parts.next()?;
    let filesystem_type = parts.next()?;
    let mount_options = parts.next()?;
    // The expected device type and fs type is "cgroup";
    if filesystem_type != b"cgroup" {
        return None;
    }

    if !mount_options
        .split(|byte| *byte == b',')
        .any(|key| key == b"net_cls")
    {
        return None;
    }

    Some(PathBuf::from(OsStr::from_bytes(mount_path)))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_find_net_cls_path() {
        let input =
            br#"cgroup /sys/fs/cgroup/memory cgroup rw,nosuid,nodev,noexec,relatime,memory 0 0
cgroup /sys/fs/cgroup/net_cls,net_prio cgroup rw,nosuid,nodev,noexec,relatime,net_cls,net_prio 0 0
"#;

        assert_eq!(
            find_net_cls_mount_inner(input),
            Some(PathBuf::from("/sys/fs/cgroup/net_cls,net_prio"))
        )
    }

    #[test]
    fn test_fail_to_find_net_cls_path() {
        let input =
            br#"cgroup /sys/fs/cgroup/memory cgroup rw,nosuid,nodev,noexec,relatime,memory 0 0
cgroup /sys/fs/cgroup/net_cls,net_prio cgroup rw,nosuid,nodev,noexec,relatime,,net_prio 0 0
cgroup /sys/fs/cgroup/net_cls,net_prio cgroup2 rw,nosuid,nodev,noexec,relatime,net_cls,net_prio 0 0
cgroup /sys/fs/cgroup/net_cls,net_prio garbage rw,nosuid,nodev,noexec,relatime,net_cls,net_prio 0 0
cgroup /nope
"#;

        assert_eq!(find_net_cls_mount_inner(input), None)
    }
}
