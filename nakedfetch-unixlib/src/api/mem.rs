use core::ffi::c_void;

use crate::{
    errno::{Errno, ErrnoResult},
    syscall,
    table::Table,
};

pub mod prot {
    pub const READ: u32 = 0b1;
    pub const WRITE: u32 = 0b10;
    pub const EXEC: u32 = 0b100;
}

pub mod flags {
    pub const SHARED: u32 = 0b1;
    pub const PRIVATE: u32 = 0b10;
    pub const SHARED_VALIDATE: u32 = 0b11;

    pub const FIXED: u32 = 0b10000;
    pub const ANONYMOUS: u32 = 0b100_000;

    pub const BIT32: u32 = 0x40;
    pub const GROWSDOWN: u32 = 0x0100;
    pub const LOCKED: u32 = 0x2000;
    pub const NORESERVE: u32 = 0x4000;
    pub const POPULATE: u32 = 0x8000;
    pub const NONBLOCK: u32 = 0x10000;
    pub const STACK: u32 = 0x20000;
    pub const HUGETLB: u32 = 0x40000;
    pub const SYNC: u32 = 0x80000;
    pub const FIXED_NOREPLACE: u32 = 0x100_000;
    pub const UNINITIALIZED: u32 = 0x4_000_000;
}

pub fn mmap(
    addr: *const c_void,
    len: usize,
    prot: u32,
    flags: u32,
    fd: i32,
    offset: usize,
) -> ErrnoResult<*mut c_void> {
    // SAFETY: We pass a valid nr,
    // all values are synchronized with https://syscalls.mebeim.net
    let ret = unsafe { syscall!(Table::Mmap, addr, len, prot, flags, fd, offset) };

    Errno::into_result(ret).map(|v| v as *mut c_void)
}

pub fn munmap(addr: *mut c_void, len: usize) -> ErrnoResult<()> {
    // SAFETY: We pass a valid nr,
    // all values are synchronized with https://syscalls.mebeim.net
    let ret = unsafe { syscall!(Table::Munmap, addr, len) };

    Errno::into_result(ret).map(|_| ())
}
