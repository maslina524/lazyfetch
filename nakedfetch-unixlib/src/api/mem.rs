use core::ffi::c_void;

use crate::{errno::{Errno, ErrnoResult}, syscall, table::Table};

pub mod prot {
    pub const READ: u32 = 0b1;
    pub const WRITE: u32 = 0b10;
    pub const EXEC: u32 = 0b100;
}

pub mod flags {
    pub const MAP_SHARED: u32 = 0b1;
    pub const MAP_PRIVATE: u32 = 0b10;
    pub const MAP_SHARED_VALIDATE: u32 = 0b11;

    pub const MAP_FIXED: u32 = 0b10000;
    pub const MAP_ANONYMOUS: u32 = 0b100_000;

    pub const MAP_32BIT: u32 = 0x40;
    pub const MAP_GROWSDOWN: u32 = 0x0100;
    pub const MAP_LOCKED: u32 = 0x2000;
    pub const MAP_NORESERVE: u32 = 0x4000;
    pub const MAP_POPULATE: u32 = 0x8000;
    pub const MAP_NONBLOCK: u32 = 0x10000;
    pub const MAP_STACK: u32 = 0x20000;
    pub const MAP_HUGETLB: u32 = 0x40000;
    pub const MAP_SYNC: u32 = 0x80000;
    pub const MAP_FIXED_NOREPLACE: u32 = 0x100_000;
    pub const MAP_UNINITIALIZED: u32 = 0x4_000_000;
}

pub fn mmap(addr: *const c_void, len: usize, prot: u32, flags: u32, fd: i32, offset: usize) -> ErrnoResult<*mut c_void> {
    // SAFETY: We pass a valid nr,
    // all values are synchronized with https://syscalls.mebeim.net
    let ret = unsafe { 
        syscall!(Table::Mmap, addr, len, prot, flags, fd, offset)
    };

    Errno::into_result(ret).map(|v| v as *mut c_void)
}