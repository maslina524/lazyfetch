use alloc::{
    string::String,
    borrow::ToOwned
};

use crate::{
    windows::env,
    windows::regedit::RegValue,
    windows::link::{SYSTEM_INFO, GetSystemInfo},
    formats::MemorySize,
    detect::kernel::KernelInfo,
    format
};

const SYSNAME: &str = "WIN32_NT";

impl KernelInfo {
    pub fn new() -> Self {
        let release = Self::release();
        let formated = format!("{SYSNAME} {release}");

        Self { 
            sysname: SYSNAME, 
            release, 
            version: Self::version(), 
            display_version: formated, 
            page_size: Self::page_size()
        }
    }

    fn version() -> String {
        let value = env::current_version().read("BuildLabEx").unwrap_or(RegValue::None);
        value.as_string().map_or_default(ToOwned::to_owned)
    }

    fn ubr() -> u32 {
        let value = env::current_version().read("UBR").unwrap_or(RegValue::None);
        value.as_u32().unwrap_or(0)
    }

    fn release() -> String {
        let (major, minor, build) = env::get_version();
        let ubr = Self::ubr();
        format!("{major}.{minor}.{build}.{ubr}")
    }

    fn page_size() -> MemorySize {
        let mut si = SYSTEM_INFO::default();
        // SAFETY: Completely safe
        unsafe {
            GetSystemInfo(&raw mut si);
        }

        MemorySize::from_bytes(si.dwPageSize as u64)
    }
}

#[cfg(test)]
mod tests {
    use crate::detect::kernel::KernelInfo;


    #[test]
    fn release_test() {
        let release = KernelInfo::release();
        println!("{release}");
    }

    #[test]
    fn version_test() {
        let version = KernelInfo::version();
        println!("{version}");
    }

    #[test]
    fn page_size_test() {
        let size = KernelInfo::page_size();
        println!("{size:.02} KiB");
    }
}