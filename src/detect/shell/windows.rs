use core::ffi::{c_void, CStr};

use crate::{
    modules::Shell, str::SmolStr, warning, windows::{encoding::{Utf16Len, utf16le_to_utf8}, env, error::ErrorCode, link::{OpenProcess, QueryFullProcessImageNameW}, path::Path}
};

const PROCESS_QUERY_LIMITED_INFORMATION: u32         = 0x1000;
const INVALID_HANDLE                   : *mut c_void = (-1isize).cast_unsigned() as *mut c_void;

static BLACKLIST: [&CStr; 6] = [
    c"python.exe", c"python3.exe", c"lazyfetch.exe",
    c"fastfetch.exe", c"neofetch.exe", c"cargo.exe"
];

#[derive(Default)]
struct Process {
    pid: u32,
    exe_path: Path
}

impl Process {
    pub fn new(pid: u32) -> Self {
        // SAFETY: Completely safe
        let handle = unsafe { 
            OpenProcess(
                PROCESS_QUERY_LIMITED_INFORMATION, 
                0, 
                pid
            ) 
        };
        
        if handle == INVALID_HANDLE {
            warning!("Failed to get shell process handle: {}", ErrorCode::last());
            return Self::default()
        }
        let mut size = 260;
        let mut buf = [0u16; 260];
        // SAFETY: Completely safe
        let ret = unsafe {
            QueryFullProcessImageNameW(
                handle, 
                0, 
                buf.as_mut_ptr(), 
                &raw mut size
            )
        };
    
        let exe_path = if ret == 1 {
            // SAFETY: WinAPI returns a valid C string and always leaves 261 bytes zeroed
            match utf16le_to_utf8(&buf, Utf16Len::Len(size as usize)) {
                Ok(s) => Path::from(s),
                Err(e) => {
                    warning!("Failed to convert Utf16 to Utf8 in shell exe path: {e}");
                    Path::default()
                }
            }
        } else {
            warning!("Failed to get shell exe path: {}", ErrorCode::last());
            Path::default()
        };

        Self { pid, exe_path }
    }
}

fn compute_shell_info() -> Process {
    let mut pid = env::get_my_pid();
    loop {
        pid = if let Some(p) = env::get_ppid_by_pid(pid) {
            p
        } else {
            warning!("Process ppid not foind (pid: {pid})");
            return Process::default()
        };
        let name = env::get_name_by_pid(pid).expect("Unreachable");

        if !BLACKLIST.contains(&name) {
            break;
        }
    }

    Process::new(pid)
}

pub fn get() -> Shell {
    let info = compute_shell_info();

    let exe_path = info.exe_path.clone();
    let process_name = info.exe_path
        .last()
        .map_or_default(SmolStr::from);

    let version = match env::get_file_product_version(&exe_path) {
        Ok(v) => v,
        Err(e) => {
            warning!("Failed to get file version: {e} (shell)");
            SmolStr::from_static("0.0.0.0")
        }
    };

    let pid = info.pid;
    let exe_name = process_name.clone();
    let exe = exe_path.clone();
    let pretty_name = SmolStr::from(exe_name.trim_end_matches(".exe"));

    Shell { 
        process_name,
        exe,
        exe_name,
        version,
        pid,
        pretty_name,
        exe_path,
        tty: -1
    }
}