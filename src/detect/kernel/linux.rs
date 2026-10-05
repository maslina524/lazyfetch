use alloc::{
    string::String,
    borrow::ToOwned
};

use crate::{
    detect::kernel::KernelInfo, 
    formats::MemorySize, 
    unix::fs,
    unix::libc::sysconf,
    abort
};

const SYSNAME: &str = "Linux";
const SC_PAGESIZE: i32 = 30;

impl KernelInfo {
    pub fn new() -> Self {
        let content = fs::read_to_string("/proc/version")
            .unwrap_or_else(|e| abort!("Failed to open /proc/version: {e}"));

        let mut splited = content.split(' ');
        splited.next();
        let release = splited.next().unwrap_or("Unknown").to_owned();
        
        let version = content
            .find("#1 SMP")
            .map_or("Unknown", |index| &content[index..])
            .to_owned();

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