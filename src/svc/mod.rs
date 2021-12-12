#[cfg(target_os = "windows")]
pub(crate) mod winsvc;

#[cfg(target_os = "linux")]
pub(crate) mod systemd;
