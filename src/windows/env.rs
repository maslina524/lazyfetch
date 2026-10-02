use core::{
    ffi::{c_void, CStr},
    ptr,
    mem,
    slice,
    fmt::Write
};

use alloc::{
    string::{String, ToString},
    vec::Vec,
    vec
};

use crate::{
    ARGS, 
    format, 
    sync::OnceLock, 
    w, 
    warning, 
    windows::{
        encoding::{wide, utf16le_to_utf8, Utf16Len}, 
        error::{self, ErrorCode}, 
        fs::{Access, File}, 
        link::{
            CONSOLE_SCREEN_BUFFER_INFO, CloseHandle, CommandLineToArgvW, CreateToolhelp32Snapshot, 
            EnumProcesses, FILETIME, FileTimeToLocalFileTime, FileTimeToSystemTime, GetCommandLineW, 
            GetConsoleScreenBufferInfo, GetFileVersionInfoSizeW, GetFileVersionInfoW, 
            GetSystemTimeAsFileTime, OSVERSIONINFOW, PROCESSENTRY32, Process32First, 
            Process32Next, RtlGetVersion, SYSTEMTIME, VerQueryValueW, GetEnvironmentVariableW
        }, 
        path::Path, 
        regedit::{self, Hkey, Regedit}
    }
};

const EPOCH_DIFF              : u64               = 116_444_736_000_000_000;
const EPOCH_DIFF_SECS         : u64               = 11_644_473_600;
const INVALID_HANDLE          : *mut c_void       = (-1isize).cast_unsigned() as *mut c_void;

const TICKS_PER_SEC           : u64               = 10_000_000;
const LOCALE_NAME_USER_DEFAULT: *const u16        = ptr::null();
const DEFAULT_DATE_FMT        : [u16; 11]         = w!("dd.MM.yyyy");
const TIME_FMT                : [u16; 9]          = w!("HH:mm:ss");
static TERMINAL_HANDLE        : OnceLock<isize>   = OnceLock::new();
static CURRENT_VERSION        : OnceLock<Regedit> = OnceLock::new();

#[repr(C)]
#[derive(Default, Debug)]
#[allow(non_snake_case, reason = "Copied from the Windows docs")]
// https://learn.microsoft.com/en-us/windows/win32/api/verrsrc/ns-verrsrc-vs_fixedfileinfo
struct VS_FIXEDFILEINFO {
  pub dwSignature: u32,
  pub dwStrucVersion: u32,
  pub dwFileVersionMS: u32,
  pub dwFileVersionLS: u32,
  pub dwProductVersionMS: u32,
  pub dwProductVersionLS: u32,
  pub dwFileFlagsMask: u32,
  pub dwFileFlags: u32,
  pub dwFileOS: u32,
  pub dwFileType: u32,
  pub dwFileSubtype: u32,
  pub dwFileDateMS: u32,
  pub dwFileDateLS: u32
}

pub fn current_version() -> &'static Regedit {
    CURRENT_VERSION.get_or_init(|| {
        Regedit::open(
            Hkey::LocalMachine, 
            "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion", 
            regedit::Access::Read
        ).unwrap()
    })
}

pub fn get_version() -> (u32, u32, u32) {
    let mut osvi = OSVERSIONINFOW::default();
    // SAFETY: Completely safe
    unsafe { RtlGetVersion(&raw mut osvi) };

    (
        osvi.dwMajorVersion,
        osvi.dwMinorVersion,
        osvi.dwBuildNumber
    )
}

pub fn terminal_handle() -> isize {
    *TERMINAL_HANDLE.get_or_init(|| {
        let file = File::open("CONOUT$", Access::Read).unwrap();
        let handle = file.as_handle();
        assert!(!handle.is_null());
        mem::forget(file);
        handle as isize
    })
}

pub fn terminal_size() -> (usize, usize) {
    let mut buf = CONSOLE_SCREEN_BUFFER_INFO::default();

    // SAFETY: Comletely safe
    let ret = unsafe {
        GetConsoleScreenBufferInfo(
            terminal_handle() as *mut c_void, 
            &raw mut buf
        )
    };
    if ret == 0 {
        ErrorCode::last().panic();
    }

    let w = buf.srWindow.Right - buf.srWindow.Left + 1;
    let h = buf.srWindow.Bottom - buf.srWindow.Top + 1;
    (w as usize, h as usize)
}

pub fn close_terminal_handle() -> error::Result<()> {
    // SAFETY: Completely safe
    let ret = unsafe {
        CloseHandle(terminal_handle() as *mut c_void)
    };
    if ret == 0 {
        return Err(ErrorCode::last());
    }
    Ok(())
}

pub fn processes_count() -> usize {
    let mut pids = [0u32; 1024];
    let mut needed = 0;

    // SAFETY: Completely safe
    let ret = unsafe {
        EnumProcesses(
            pids.as_mut_ptr(), 
            (pids.len() * 4) as u32, 
            &raw mut needed
        )
    };
    if ret == 0 {
        ErrorCode::last().panic();
    }
    needed as usize / 4
}

pub fn timestamp_mils() -> u64 {
    let mut info = FILETIME::default();

    // SAFETY: Completely safe
    unsafe {
        GetSystemTimeAsFileTime(&raw mut info);
    };

    let uli = ((info.dwHighDateTime as u64) << 32) | (info.dwLowDateTime as u64);
    (uli - EPOCH_DIFF) / 10_000
}

pub fn timestamp_secs() -> u64 {
    timestamp_mils() / 1_000
}

pub fn timestamp_hours() -> u64 {
    timestamp_secs() / 3600
}

pub fn args_init() -> Vec<String> {
    // SAFETY: Completely safe
    let ptr = unsafe { GetCommandLineW() };
    let mut argv_count = 0;

    // SAFETY: Just a WinAPI function,
    // I don't know what to write, it's safe
    let argv_ptrs = unsafe {
        CommandLineToArgvW(
            ptr, 
            &raw mut argv_count
        )
    };

    let mut ret = Vec::with_capacity(argv_count as usize);
    for i in 0..argv_count {
        // SAFETY: Moving strictly within the allocated memory by WinAPI
        let start_ptr = unsafe { *argv_ptrs.add(i as usize) };

        let mut len = 0;
        // SAFETY: Moving until the end of the allocated string
        while unsafe { *start_ptr.add(len) } != 0 {
            len += 1;
        }

        // SAFETY: All parameters were correctly calculated earlier.
        let slice = unsafe { slice::from_raw_parts(start_ptr, len) };
        let string = String::from_utf16_lossy(slice);
        ret.push(string);
    }

    ret
}

pub fn args() -> &'static Vec<String> {
    ARGS.get().expect("Unreachable")
}

pub fn find_pid_by_name(name: &str) -> u32 {
    let mut pid = 0;
    // SAFETY: Completely safe
    let snapshot = unsafe { CreateToolhelp32Snapshot(2, 0) };
    if snapshot == INVALID_HANDLE { return 0; }

    let mut pe = PROCESSENTRY32 {
        dwSize: size_of::<PROCESSENTRY32>() as u32,
        .. PROCESSENTRY32::default()
    };

    // SAFETY: Completely safe
    let first = unsafe { Process32First(snapshot, &raw mut pe) };
    if first != 0 {
        loop {
            // SAFETY: Libc is guaranteed to return a valid c string
            let proc_name = unsafe { CStr::from_ptr(pe.szExeFile.as_ptr()) };
            if proc_name.to_bytes() == name.as_bytes() {
                pid = pe.th32ProcessID;
                break;
            }
            // SAFETY: Completely safe
            let ret = unsafe { Process32Next(snapshot, &raw mut pe) };
            if ret == 0 {
                break;
            }
        }
    }

    // SAFETY: Completely safe
    unsafe { CloseHandle(snapshot) };
    pid
}

pub fn get_file_product_version(path: impl Into<Path>) -> error::Result<String> {
    let path_str = path.into().into_inner();
    let path_wide = wide(path_str)?;

    // SAFETY: Completely safe
    let buf_size = unsafe { 
        GetFileVersionInfoSizeW(
            path_wide.as_ptr(), 
            ptr::null_mut()
        ) 
    };
    if buf_size == 0 {
        return Err(ErrorCode::last());
    }

    let mut buf = vec![0u8; buf_size as usize];
    // SAFETY: Completely safe
    let ret = unsafe {
        GetFileVersionInfoW(
            path_wide.as_ptr(),
            0,
            buf_size,
            buf.as_mut_ptr().cast(),
        )
    };
    if ret == 0 {
        return Err(ErrorCode::last());
    }

    let mut info_ptr: *mut c_void = ptr::null_mut();
    let mut len = 0;
    let wide_subblock = wide("\\")?;

    // SAFETY: Completely safe
    let ret = unsafe {
        VerQueryValueW(
            buf.as_ptr().cast(),
            wide_subblock.as_ptr(),
            &raw mut info_ptr,
            &raw mut len,
        )
    };
    if ret == 0 {
        return Err(ErrorCode::last());
    }

    // SAFETY: A guaranteed non-null pointer
    let info = unsafe { &*(info_ptr as *const VS_FIXEDFILEINFO) };

    let major = (info.dwProductVersionMS >> 16) & 0xFFFF;
    let minor = info.dwProductVersionMS & 0xFFFF;
    let build = (info.dwProductVersionLS >> 16) & 0xFFFF;
    let rev = info.dwProductVersionLS & 0xFFFF;

    Ok(format!("{major}.{minor}.{build}.{rev}"))
}

pub fn format_timestamp(time: u64, format: Option<&str>) -> String {
    let Some(ticks) = time
        .checked_add(EPOCH_DIFF_SECS)
        .and_then(|s| s.checked_mul(TICKS_PER_SEC)) else {
            warning!("timestamp overflow: {}", time);
            return time.to_string();
        };

    let ft_utc = FILETIME {
        dwLowDateTime:  (ticks & 0xFFFF_FFFF) as u32,
        dwHighDateTime: (ticks >> 32) as u32,
    };

    let mut ft_local = FILETIME::default();
    let mut st = SYSTEMTIME::default();

    // SAFETY: Completely safe
    let ret = unsafe { 
        FileTimeToLocalFileTime(&raw const ft_utc, &raw mut ft_local) 
    };
    if ret == 0 {
        warning!("Failed to call FileTimeToLocalFileTime: {}", ErrorCode::last());
        return time.to_string();
    }

    // SAFETY: Completely safe
    let ret = unsafe { 
        FileTimeToSystemTime(&raw const ft_local, &raw mut st) 
    };
    if ret == 0 {
        warning!("Failed to call FileTimeToSystemTime: {}", ErrorCode::last());
        return time.to_string();
    }

    format_system_time(&st, format.unwrap_or("%Y-%m-%d %H:%M:%S"))
}

fn format_system_time(st: &SYSTEMTIME, fmt: &str) -> String {
    let mut out = String::with_capacity(fmt.len() + 16);
    let mut chars = fmt.chars();

    while let Some(c) = chars.next() {
        if c != '%' {
            out.push(c);
            continue;
        }

        let Some(spec) = chars.next() else {
            out.push('%');
            break;
        };

        match spec {
            'Y' => { let _ = write!(out, "{:04}", st.wYear); }
            'y' => { let _ = write!(out, "{:02}", st.wYear % 100); }
            'm' => { let _ = write!(out, "{:02}", st.wMonth); }
            'd' => { let _ = write!(out, "{:02}", st.wDay); }
            'e' => { let _ = write!(out, "{:2}",  st.wDay); }
            'j' => { let _ = write!(out, "{:03}", day_of_year(st)); }

            'H' => { let _ = write!(out, "{:02}", st.wHour); }
            'I' => {
                let h = st.wHour % 12;
                let h = if h == 0 { 12 } else { h };
                let _ = write!(out, "{h:02}");
            }
            'M' => { let _ = write!(out, "{:02}", st.wMinute); }
            'S' => { let _ = write!(out, "{:02}", st.wSecond); }
            'f' => { let _ = write!(out, "{:03}", st.wMilliseconds); }

            '%' => out.push('%'),
            other => {
                out.push('%');
                out.push(other);
            }
        }
    }

    out
}

#[inline]
fn month_idx(st: &SYSTEMTIME) -> usize {
    (st.wMonth.saturating_sub(1) as usize).min(11)
}

fn day_of_year(st: &SYSTEMTIME) -> u16 {
    const CUM: [u16; 12] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let mut d = CUM[month_idx(st)];
    if st.wMonth > 2 && is_leap_year(st.wYear) {
        d += 1;
    }
    d + st.wDay
}

const fn is_leap_year(year: u16) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

pub fn get_var(name: &str, size: Option<usize>) -> error::Result<String> {
    let name_wide = wide(name)?;
    let size = size.unwrap_or(1024);
    let mut buf = vec![0u16; size + 1];

    // SAFETY: Buffer overflow will not occur,
    //  1 byte remains for the null byte, safe
    let len = unsafe {
        GetEnvironmentVariableW(
            name_wide.as_ptr(),
            buf.as_mut_ptr(),
            size as u32
        )
    };
    if len == 0 {
        return Err(ErrorCode::last());
    }

    utf16le_to_utf8(&buf, Utf16Len::Len(len as usize))
}

#[cfg(test)]
mod tests {
    use crate::windows::env;
  
    extern crate std;

    #[test]
    fn timestamp_secs_test() {
        for _ in 0..100_000 {
            let timestamp = env::timestamp_secs();
            println!("{timestamp}");
        }
    }

    #[test]
    fn terminal_size_test() {
        let size = env::terminal_size();
        println!("{size:?}");
    }

    #[test]
    fn find_pid_test() {
        let name = "System";
        let pid = env::find_pid_by_name(name);
        assert_eq!(pid, 4);
    }

    #[test]
    fn file_version_test() {
        let path = "C:/Windows/System32/smss.exe";
        let version = env::get_file_product_version(path);
        println!("{version:#?}");
    }
}