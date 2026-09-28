use alloc::{
    borrow::Cow, 
    string::String
};

use crate::{
    imp::path::Path,
    cfg_if
};

cfg_if! {
    if #[cfg(target_os = "windows")] {
        mod windows;
    } else if #[cfg(target_os = "linux")] {
        mod linux;
    } else if #[cfg(target_os = "android")] {
        mod android;
    }
}

pub struct InitSystemInfo {
    pub exe: Path,
    pub name: Cow<'static, str>,
    pub version: String
}

#[cfg(target_os = "windows")]
pub fn pid() -> u32 {
    crate::imp::env::find_pid_by_name("smss.exe")
}

#[cfg(not(target_os = "windows"))]
pub const fn pid() -> u32 { 1 }