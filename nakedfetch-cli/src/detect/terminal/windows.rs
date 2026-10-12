use core::ffi::c_void;

use crate::{
    detect::shell::get_shell_pid,
    modules::Terminal,
    str::SmolStr,
    warning,
    windows::{
        encoding::{Utf16Len, utf16le_to_utf8},
        env,
        error::ErrorCode,
        link::{OpenProcess, QueryFullProcessImageNameW},
        path::Path,
    },
};

const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;
const INVALID_HANDLE: *mut c_void = (-1isize).cast_unsigned() as *mut c_void;

#[derive(Default)]
struct Process {
    pid: u32,
    exe_path: Path,
}

impl Process {
    pub fn new(pid: u32) -> Self {
        // SAFETY: Completely safe
        let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };

        if handle == INVALID_HANDLE {
            warning!(
                "Failed to get terminal process handle: {}",
                ErrorCode::last()
            );
            return Self::default();
        }
        let mut size = 260;
        let mut buf = [0u16; 260];
        // SAFETY: Completely safe
        let ret = unsafe { QueryFullProcessImageNameW(handle, 0, buf.as_mut_ptr(), &raw mut size) };

        let exe_path = if ret == 1 {
            // SAFETY: WinAPI returns a valid C string and always leaves 261 bytes zeroed
            match utf16le_to_utf8(&buf, Utf16Len::Len(size as usize)) {
                Ok(s) => Path::from(s),
                Err(e) => {
                    warning!("Failed to convert Utf16 to Utf8 in terminal exe path: {e}");
                    Path::default()
                }
            }
        } else {
            warning!("Failed to get terminal exe path: {}", ErrorCode::last());
            Path::default()
        };

        Self { pid, exe_path }
    }
}

pub fn get() -> Terminal {
    let Some(pid) = env::get_ppid_by_pid(get_shell_pid()) else {
        return Terminal::default();
    };
    let info = Process::new(pid);

    let exe_path = info.exe_path.clone();
    let process_name = info.exe_path.last().map_or_default(SmolStr::from);

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

    Terminal {
        process_name,
        exe,
        exe_name,
        version,
        pid,
        pretty_name,
        exe_path,
        tty: -1,
    }
}
