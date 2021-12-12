mod error;
pub(crate) mod platform_specific;

#[cfg(target_os = "linux")]
mod netlink;
