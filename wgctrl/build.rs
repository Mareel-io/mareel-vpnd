// SPDX-FileCopyrightText: 2022 Empo Inc.
//
// SPDX-License-Identifier: CC0-1.0

use std::{env, path::PathBuf};

fn manifest_dir() -> PathBuf {
    env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .expect("CARGO_MANIFEST_DIR env var not set")
}

fn main() {
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();

    let windns = manifest_dir().join("../windns/x64-Release");

    match (
        target_arch.as_str(),
        target_os.as_str(),
        target_env.as_str(),
    ) {
        (_arch, "windows", "msvc") => {
            println!("cargo:rustc-link-search={}", &windns.display());
            println!("cargo:rustc-link-lib=static=windns");
        }
        (_arch, "windows", target_env) => {
            panic!("Sorry, env {} is not supported on Windows", target_env);
        }
        (_, _, _) => {}
    }
}
