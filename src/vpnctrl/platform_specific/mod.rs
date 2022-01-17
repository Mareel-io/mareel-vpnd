use self::common::{PlatformError, PlatformInterface, PlatformRoute};

// Platform common
pub(crate) mod common;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub(crate) use windows::*;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub(crate) use macos::*;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub(crate) use linux::*;

use super::error::VpnctrlError;

pub struct PlatformSpecificFactory;

impl PlatformSpecificFactory {
    pub fn get_interface(name: &str) -> Result<Interface, VpnctrlError> {
        Interface::new(name)
    }

    pub fn get_route(fwmark: u32) -> Result<Route, VpnctrlError> {
        Route::new(fwmark)
    }
}
