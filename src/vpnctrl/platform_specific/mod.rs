use self::common::{PlatformError, PlatformInterface};

// Platform common
pub(crate) mod common;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
mod linux;

pub struct PlatformSpecificFactory;

impl PlatformSpecificFactory {
    #[cfg(target_os = "windows")]
    fn get_interface(name: &str) -> Result<windows::Interface, PlatformError> {
        windows::Interface::new(name)
    }

    #[cfg(target_os = "macos")]
    fn get_interface(name: &str) -> Result<macos::Interface, PlatformError> {
        Err(PlatformError::new("Not supported yet :(".to_string()))
    }

    #[cfg(target_os = "linux")]
    fn get_interface(name: &str) -> Result<linux::Interface, PlatformError> {
        Err(PlatformError::new("Not supported yet :(".to_string()))
    }
}
