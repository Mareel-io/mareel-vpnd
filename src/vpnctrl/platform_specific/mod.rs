use self::{common::{PlatformError, PlatformInterface}, windows::Interface};

// Platform common
mod common;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
mod linux; 

pub struct PlatformSpecificFactory;

impl PlatformSpecificFactory {
    #[cfg(target_os = "windows")]
    fn get_interface (name: &str) -> Result<windows::Interface, PlatformError> {
        windows::Interface::new(name)
    }

    #[cfg(target_os = "macos")]
    fn get_interface (name: &str) -> Result<macos::Interface, PlatformError> {
        PlatformError::new("Not supported yet :(");
    }

    #[cfg(target_os = "linux")]
    fn get_interface (name: &str) -> Result<linux::Interface, PlatformError> {
        PlatformError::new("Not supported yet :(");
    }
}
