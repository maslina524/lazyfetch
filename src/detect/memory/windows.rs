use core::mem;

use crate::{
    detect::memory::MemoryInfo,
    formats::MemorySize,
    windows::link::{GlobalMemoryStatusEx, MEMORYSTATUSEX},
};

impl MemoryInfo {
    pub fn new() -> Self {
        let mut mem_status = MEMORYSTATUSEX {
            dwLength: mem::size_of::<MEMORYSTATUSEX>() as u32,
            ..MEMORYSTATUSEX::default()
        };

        // SAFETY: Completely safe
        let ret = unsafe { GlobalMemoryStatusEx(&raw mut mem_status) };
        if ret == 0 {
            return Self::default();
        }

        let total_raw = mem_status.ullTotalPhys;
        let in_use_raw = mem_status.ullTotalPhys - mem_status.ullAvailPhys;

        let total = MemorySize::from_bytes(total_raw);
        let in_use = MemorySize::from_bytes(in_use_raw);

        Self { total, in_use }
    }
}

#[cfg(test)]
mod tests {
    use crate::{detect::memory::MemoryInfo, formats::MemorySize};

    #[test]
    fn ram_test() {
        let mem = MemoryInfo::new();
        let total = mem.total;
        let in_use = mem.in_use;

        println!("Total: {total}, In use: {in_use}");

        assert_ne!(total, MemorySize::default());
        assert_ne!(in_use, MemorySize::default());
    }
}
