use core::ffi::CStr;

use alloc::{
    string::String,
    borrow::ToOwned
};

use crate::{
    detect::kernel::KernelInfo, 
    formats::MemorySize, 
    unix::{
        libc::{Utsname, sysconf, uname}
    }
};

const SYSNAME: &str = "Linux";
const SC_PAGESIZE: i32 = 30;

impl KernelInfo {
    pub fn new() -> Self {
        let mut info = Utsname::default();
        let (release, version) = if uname(&raw mut info) == 0 {
            let release_raw = CStr::from_bytes_until_nul(&info.release)
                .unwrap()
                .to_string_lossy();

            let release = if let Some(index) = release_raw.find('-') {
                release_raw[..index].to_owned()
            } else {
                release_raw.into_owned()
            };

            let version = CStr::from_bytes_until_nul(&info.version)
                .unwrap()
                .to_string_lossy()
                .into_owned();

            (release, version)
        } else {
            ("Unknown".to_owned(), "Unknown".to_owned())
        };

        Self { 
            sysname: SYSNAME, 
            release, 
            version, 
            display_version: String::new(), 
            page_size: Self::page_size()
        }
    }

    fn page_size() -> MemorySize {
        let bytes = sysconf(SC_PAGESIZE);
        MemorySize::from_bytes(bytes as u64)
    }
}