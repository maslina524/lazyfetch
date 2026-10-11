use alloc::borrow::Cow;

use crate::{cfg_if, imp::path::Path, str::SmolStr};

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
    pub version: SmolStr,
}

#[cfg(target_os = "windows")]
pub fn pid() -> u32 {
    crate::imp::env::get_initsystem_pid()
}

#[cfg(not(target_os = "windows"))]
pub const fn pid() -> u32 {
    1
}
