use alloc::string::String;

use crate::formats::MemorySize;

crate::cfg_if! {
    if #[cfg(target_os = "windows")] {
        mod windows;
    } else if #[cfg(target_os = "linux")] {
        mod linux;
    } else if #[cfg(target_os = "android")] {
        mod android;
    }
}

pub struct KernelInfo {
    pub sysname: &'static str,
    pub release: String,
    pub version: String,
    pub display_version: String,
    pub page_size: MemorySize,
}
