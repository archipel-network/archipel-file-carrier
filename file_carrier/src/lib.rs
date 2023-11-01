use std::{path::PathBuf, str::FromStr};

pub mod error;
pub mod unregister;
pub mod register;
pub mod init;
pub mod hierarchy;

extern "C" {
    fn geteuid() -> u32;
}

#[inline]
pub fn default_path() -> PathBuf {
    unsafe {
        PathBuf::from_str(&format!("/run/user/{}/archipel-core/archipel-core.socket", geteuid())).unwrap()
    }
}
