#![cfg(windows)]
pub mod graphics;
pub mod metrics;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub fn wide(value: impl AsRef<std::ffi::OsStr>) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    value.as_ref().encode_wide().chain(Some(0)).collect()
}
