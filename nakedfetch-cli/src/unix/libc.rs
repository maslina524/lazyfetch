#![allow(
    non_camel_case_types,
    non_snake_case,
    clippy::struct_field_names,
    suspicious_runtime_symbol_definitions,
    invalid_runtime_symbol_definitions,
    reason = "C type"
)]

use core::ffi::{CStr, c_char, c_int, c_long, c_uchar, c_uint, c_ulong, c_ushort, c_void};

use crate::sync::OnceLock;

pub type FileHandle = *mut c_void;
pub type Dir = *mut c_void;

pub type c_size = usize;
pub type c_ssize = isize;
pub type c_mode = c_uint;
pub type c_time = c_long; // i64
pub type c_clockid = c_int; // WORK ONLY IN LINUX
pub type c_uid = u32;
pub type c_gid = u32;
pub type c_pid = i32;
pub type c_socklen = c_uint;
pub type c_sa_family = c_ushort;
pub type c_fsblkcnt = c_ulong;
pub type c_fsfilcnt = c_ulong;
pub type c_dev = c_ulong;
pub type c_nlink = c_ulong;
pub type c_blksize = c_long;

pub type c_ino = usize;
pub type c_off = isize;
pub type c_blkcnt = isize;

#[cfg(target_arch = "x86_64")]
const SYS_GETDENTS64: c_long = 217;
#[cfg(target_arch = "aarch64")]
const SYS_GETDENTS64: c_long = 61;

#[cfg(target_arch = "x86_64")]
const SYS_STATX: c_long = 332;
#[cfg(target_arch = "aarch64")]
const SYS_STATX: c_long = 291;

#[link(name = "c")]
unsafe extern "C" {
    pub safe fn write(fd: i32, buf: *const c_void, len: c_size) -> c_ssize;
    pub unsafe fn malloc(size: c_size) -> *mut c_void;
    pub unsafe fn free(ptr: *mut c_void);
    pub unsafe fn calloc(nmemb: c_size, size: c_size) -> *mut c_void;
    pub unsafe fn realloc(ptr: *mut c_void, size: c_size) -> *mut c_void;
    pub unsafe fn exit(status: c_int) -> !;
    pub safe fn getenv(name: *const c_char) -> *mut c_char;
    pub safe fn access(path: *const c_char, mode: c_int) -> c_int;
    pub safe fn fopen(pathname: *const c_char, mode: *const c_char) -> FileHandle;
    pub safe fn setmntent(filename: *const c_char, typ: *const c_char) -> FileHandle;
    pub safe fn getmntent(stream: FileHandle) -> *mut Mntent;
    pub safe fn fclose(stream: FileHandle) -> c_int;
    pub safe fn strerror(errnum: c_int) -> *mut c_char;
    pub safe fn fwrite(
        ptr: *const c_void,
        size: c_size,
        nmemb: c_size,
        stream: FileHandle,
    ) -> c_size;
    pub safe fn fread(ptr: *mut c_void, size: c_size, nmemb: c_size, stream: FileHandle) -> c_size;
    pub safe fn rewind(stream: FileHandle);
    pub safe fn mkdir(pathname: *const c_char, mode: c_mode) -> c_int;
    pub safe fn clock_gettime(clockid: c_clockid, tp: *mut Timespec) -> c_int;
    pub safe fn sysinfo(info: *mut Sysinfo) -> c_int;
    pub safe fn opendir(path: *const c_char) -> Dir;
    pub safe fn readdir(dirp: *mut Dir) -> *mut Dirent;
    pub safe fn setlocale(category: c_int, locale: *const c_char) -> *mut c_char;
    pub safe fn getuid() -> c_uid;
    pub safe fn getpwuid(uid: c_uid) -> *const Passwd;
    pub safe fn gethostname(name: *mut c_char, len: c_size) -> c_int;
    pub safe fn readlink(pathname: *const c_char, buf: *mut c_char, bufsiz: c_size) -> c_ssize;
    pub safe fn getpid() -> c_pid;
    pub safe fn getppid() -> c_pid;
    pub safe fn getcwd(buf: *mut c_char, size: c_size) -> *mut c_char;
    pub safe fn dlopen(filename: *const c_char, flags: c_int) -> *mut c_void;
    pub safe fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    pub safe fn dlclose(handle: *mut c_void) -> c_int;
    pub safe fn dlerror() -> *mut c_char;
    pub safe fn sysconf(name: c_int) -> c_long;
    pub safe fn localtime(timep: *const c_time) -> *mut Tm;
    pub safe fn time(tloc: *mut c_time) -> c_time;
    pub safe fn getaddrinfo(
        node: *const c_char,
        service: *const c_char,
        hints: *const AddrInfo,
        res: *mut *mut AddrInfo,
    ) -> c_int;
    pub safe fn socket(domain: c_int, type_: c_int, protocol: c_int) -> c_int;
    pub safe fn connect(sockfd: c_int, addr: *const SockAddr, addrlen: c_socklen) -> c_int;
    pub safe fn send(sockfd: c_int, buf: *const c_void, len: c_size, flags: c_int) -> c_ssize;
    pub safe fn recv(sockfd: c_int, buf: *mut c_void, len: c_size, flags: c_int) -> c_ssize;
    pub safe fn close(fd: c_int) -> c_int;
    pub safe fn freeaddrinfo(res: *mut AddrInfo);
    pub safe fn localtime_r(timep: *const c_time, result: *mut Tm) -> *mut Tm;
    pub safe fn ferror(stream: FileHandle) -> c_int;
    pub safe fn syscall(num: c_long, ...) -> c_long;
    pub safe fn open(pathname: *const c_char, flags: c_int, mode: c_mode) -> c_int;
    pub safe fn ioctl(fd: c_int, op: c_ulong, ...) -> c_int;
    pub safe fn popen(command: *const c_char, typ: *const c_char) -> FileHandle;
    pub safe fn pclose(stream: FileHandle) -> c_int;
    pub safe fn fgets(s: *mut c_char, size: c_int, stream: FileHandle) -> *mut c_char;
    pub safe fn uname(st: *mut Utsname) -> c_int;
    pub safe fn statvfs(path: *const c_char, buf: *mut Statvfs) -> c_int;
    pub safe fn strftime(
        s: *mut c_char,
        max: c_size,
        format: *const c_char,
        tm: *const Tm,
    ) -> c_size;
    pub safe fn gmtime(timep: *const c_time) -> *mut Tm;
}

#[cfg(glibc)]
#[link(name = "c")]
unsafe extern "C" {
    pub safe fn __errno_location() -> *mut c_int;
}

#[cfg(bionic)]
#[link(name = "c")]
unsafe extern "C" {
    pub safe fn __errno() -> *mut c_int;
    pub safe fn __system_property_get(name: *const c_char, value: *mut c_char) -> c_int;
}

pub fn getdents64(fd: c_int, dirp: *mut c_void, count: c_size) -> c_ssize {
    syscall(SYS_GETDENTS64, fd, dirp, count) as c_ssize
}

pub fn statx(dfd: i32, filename: &CStr, flags: u32, mask: u32, buffer: *mut Statx) -> c_ssize {
    syscall(SYS_STATX, dfd, filename.as_ptr(), flags, mask, buffer) as c_ssize
}

static SYSINFO: OnceLock<Sysinfo> = OnceLock::new();

pub fn get_sysinfo() -> &'static Sysinfo {
    SYSINFO.get_or_init(|| {
        let mut info = Sysinfo::default();
        sysinfo(&raw mut info);
        info
    })
}

#[cfg(glibc)]
pub fn errno() -> i32 {
    // SAFETY: Libc is guaranteed to return a valid pointer
    unsafe { *__errno_location() }
}

#[cfg(bionic)]
pub fn errno() -> i32 {
    // SAFETY: Libc is guaranteed to return a valid pointer
    unsafe { *__errno() }
}

#[repr(C)]
#[derive(Default)]
pub struct Statx {
    pub stx_mask: u32,
    pub stx_blksize: u32,
    pub stx_attributes: u64,
    pub stx_nlink: u32,
    pub stx_uid: u32,
    pub stx_gid: u32,
    pub stx_mode: u16,
    pub __spare0: u16,
    pub stx_ino: u64,
    pub stx_size: u64,
    pub stx_blocks: u64,
    pub stx_attributes_mask: u64,
    pub stx_atime: StatxTimestamp,
    pub stx_btime: StatxTimestamp,
    pub stx_ctime: StatxTimestamp,
    pub stx_mtime: StatxTimestamp,
    pub stx_rdev_major: u32,
    pub stx_rdev_minor: u32,
    pub stx_dev_major: u32,
    pub stx_dev_minor: u32,
    pub stx_mnt_id: u64,
    pub stx_dio_mem_align: u32,
    pub stx_dio_offset_align: u32,
    pub stx_subvol: u64,
    pub stx_atomic_write_unit_min: u32,
    pub stx_atomic_write_unit_max: u32,
    pub stx_atomic_write_segments_max: u32,
    pub stx_dio_read_offset_align: u32,
    pub stx_atomic_write_unit_max_opt: u32,
    pub __spare1: u32,
    pub __spare2: [u64; 8],
}

#[repr(C)]
#[derive(Default)]
pub struct StatxTimestamp {
    pub tv_sec: u64,
    pub tv_nsec: u32,
}

#[repr(C)]
#[derive(Default)]
pub struct Statvfs {
    pub f_bsize: c_ulong,
    pub f_frsize: c_ulong,
    pub f_blocks: c_fsblkcnt,
    pub f_bfree: c_fsblkcnt,
    pub f_bavail: c_fsblkcnt,
    pub f_files: c_fsfilcnt,
    pub f_ffree: c_fsfilcnt,
    pub f_favail: c_fsfilcnt,
    pub f_fsid: c_ulong,
    pub f_flag: c_ulong,
    pub f_namemax: c_ulong,
    __f_spare: [c_uint; 6],
}

#[repr(C)]
#[derive(Default)]
pub struct Mntent {
    pub mnt_fsnameL: *const c_char,
    pub mnt_dirL: *const c_char,
    pub mnt_typeL: *const c_char,
    pub mnt_optsL: *const c_char,
    pub mnt_freqL: c_int,
    pub mnt_passnoL: c_int,
}

#[repr(C)]
pub struct Utsname {
    pub sysname: [u8; 65],
    pub nodename: [u8; 65],
    pub release: [u8; 65],
    pub version: [u8; 65],
    pub machine: [u8; 65],
    pub domainname: [u8; 65],
}

impl Default for Utsname {
    fn default() -> Self {
        Self {
            sysname: [0u8; 65],
            nodename: [0u8; 65],
            release: [0u8; 65],
            version: [0u8; 65],
            machine: [0u8; 65],
            domainname: [0u8; 65],
        }
    }
}

#[repr(C)]
#[derive(Default)]
pub struct Winsize {
    pub ws_row: c_ushort,
    pub ws_col: c_ushort,
    pub ws_xpixel: c_ushort,
    pub ws_ypixel: c_ushort,
}

#[repr(C)]
pub struct LinuxDirent64 {
    pub d_ino: u64,
    pub d_off: i64,
    pub d_reclen: u16,
    pub d_type: u8,
    pub d_name: [c_char; 256],
}

impl Default for LinuxDirent64 {
    fn default() -> Self {
        Self {
            d_ino: 0,
            d_off: 0,
            d_reclen: 0,
            d_type: 0,
            d_name: [c_char::default(); 256],
        }
    }
}

#[cfg(glibc)]
#[repr(C)]
#[derive(Default)]
pub struct AddrInfo {
    pub ai_flags: c_int,
    pub ai_family: c_int,
    pub ai_socktype: c_int,
    pub ai_protocol: c_int,
    pub ai_addrlen: c_socklen,
    pub ai_addr: *mut SockAddr,
    pub ai_canonname: *mut c_char,
    pub ai_next: *mut Self,
}

#[cfg(bionic)]
#[repr(C)]
#[derive(Default)]
pub struct AddrInfo {
    pub ai_flags: c_int,
    pub ai_family: c_int,
    pub ai_socktype: c_int,
    pub ai_protocol: c_int,
    pub ai_addrlen: c_uint,
    pub ai_canonname: *mut c_char,
    pub ai_addr: *mut SockAddr,
    pub ai_next: *mut Self,
}

#[repr(C)]
#[derive(Default)]
pub struct InAddr {
    pub s_addr: c_uint,
}

#[repr(C)]
#[derive(Default)]
pub struct SockAddr {
    pub sa_family: c_sa_family,
    pub sa_data: [c_char; 14],
}

#[repr(C)]
#[derive(Default)]
pub struct SockAddrIn {
    pub sin_family: c_ushort,
    pub sin_port: c_ushort,
    pub sin_addr: InAddr,
    pub sin_zero: [c_uchar; 8],
}

#[repr(C)]
#[derive(Default)]
pub struct Tm {
    pub tm_sec: c_int,
    pub tm_min: c_int,
    pub tm_hour: c_int,
    pub tm_mday: c_int,
    pub tm_mon: c_int,
    pub tm_year: c_int,
    pub tm_wday: c_int,
    pub tm_yday: c_int,
    pub tm_isdst: c_int,
    pub tm_gmtoff: c_long,
    pub tm_zone: *const c_char,
}

#[repr(C)]
#[derive(Default)]
pub struct Passwd {
    pub pw_name: *mut c_char,
    pub pw_passwd: *mut c_char,
    pub pw_uid: c_uid,
    pub pw_gid: c_gid,
    pub pw_gecos: *mut c_char,
    pub pw_dir: *mut c_char,
    pub pw_shell: *mut c_char,
}

#[repr(C)]
#[derive(Default)]
pub struct Timespec {
    pub tv_sec: c_time,
    pub tv_nsec: c_long,
}

#[repr(C)]
#[derive(Default)]
pub struct Sysinfo {
    pub uptime: c_long,
    pub loads: [c_ulong; 3],
    pub totalram: c_ulong,
    pub freeram: c_ulong,
    pub sharedram: c_ulong,
    pub bufferram: c_ulong,
    pub totalswap: c_ulong,
    pub freeswap: c_ulong,
    pub procs: c_ushort,
    pub totalhigh: c_ulong,
    pub freehigh: c_ulong,
    pub mem_unit: c_uint,
    _f: [c_char; 20 - 2 * size_of::<c_long>() - size_of::<c_int>()],
}

#[repr(C)]
pub struct Dirent {
    pub d_ino: c_ino,
    pub d_off: c_off,
    pub d_reclen: c_ushort,
    pub d_type: c_uchar,
    pub d_name: [c_char; 256],
}

impl Default for Dirent {
    fn default() -> Self {
        Self {
            d_ino: Default::default(),
            d_off: Default::default(),
            d_reclen: Default::default(),
            d_type: Default::default(),
            d_name: [0; 256],
        }
    }
}
