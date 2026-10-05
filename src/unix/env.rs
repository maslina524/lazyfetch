use core::{
    ffi::{CStr, c_char},
    slice
};

use alloc::{
    ffi::CString, 
    string::{String, ToString}, 
    vec::Vec
};

use crate::{
    ARGS, 
    abort, 
    format, 
    unix::{
        fs, 
        libc::{
            Timespec, Winsize, clock_gettime, getenv, 
            gmtime, ioctl, localtime, strftime
        }
    }, 
    warning
};

const TIOCGWINSZ: u64 = 0x5413;

#[allow(clippy::similar_names, reason = "that's what they're called in C, i don't give a fuck about clippy")]
pub fn args_init(argc: usize, argv: *const *const u8) -> Vec<String> {
    let mut ret = Vec::with_capacity(argc);
    for i in 0..argc {
        // SAFETY: Moving strictly within the allocated memory by Linux
        let start_ptr = unsafe { *argv.add(i) };

        let mut len = 0;
        // SAFETY: Moving until the end of the allocated string
        while unsafe { *start_ptr.add(len) } != 0 {
            len += 1;
        }

        // SAFETY: All parameters were correctly calculated earlier.
        let slice = unsafe { slice::from_raw_parts(start_ptr, len) };
        let string = String::from_utf8_lossy(slice).to_string();
        ret.push(string);
    }

    ret
}

pub fn args() -> &'static Vec<String> {
    ARGS.get().expect("Unreachable")
}

pub fn timestamp_mils() -> u64 {
    let mut ts = Timespec::default();
    clock_gettime(0, &raw mut ts);
    ts.tv_sec as u64 * 1000 + ts.tv_nsec as u64 / 1_000_000
}

pub fn timestamp_secs() -> u64 {
    timestamp_mils() / 1_000
}

pub fn timestamp_hours() -> u64 {
    timestamp_secs() / 3600
}

pub fn processes_count() -> usize {
    let entries = match fs::read_dir_all("/proc") {
        Ok(e) => e,
        Err(e) => {
            warning!("Failed to read /proc: {e}");
            return 0
        }
    };

    let mut ret = 0;
    for entry in entries {
        let name = entry.name();
        if name.parse::<u32>().is_ok() {
            ret += 1;
        }
    }

    ret
}

pub fn find_pid_by_name(name: &str) -> u32 {
    for entry in fs::read_dir("/proc").unwrap() {
        let entry_name = entry.name();
        let Ok(pid) = entry_name.parse::<u32>() else {
            continue;
        };

        let path = format!("/proc/{pid}/stat");
        let content = fs::read_to_string(path).unwrap();

        let lparen_index = content.find('(').unwrap_or(0);
        let rparen_index = content.find(')').unwrap_or(0);

        let process_name = &content[lparen_index + 1..rparen_index];
        if process_name == name {
            return pid;
        }
    }
    0
}

pub fn terminal_size() -> (usize, usize) {
    fn parse_env_var(name: &[u8]) -> Option<usize> {
        let name_cstr = CStr::from_bytes_with_nul(name).ok()?;
        let ptr = getenv(name_cstr.as_ptr());
        if ptr.is_null() {
            return None;
        }
        // SAFETY: Libs are guaranteed to store a valid cstr
        let cstr = unsafe { CStr::from_ptr(ptr) };
        let bytes = cstr.to_bytes();
        if bytes.is_empty() {
            return None;
        }
        let mut value = 0usize;
        for &b in bytes {
            if !b.is_ascii_digit() {
                warning!("Failed to get {} (terminal_size)", str::from_utf8(name).unwrap());
                return None;
            }
            value = value * 10 + (b - b'0') as usize;
        }
        Some(value)
    }

    let lines = parse_env_var(b"LINES\0").unwrap_or(0);
    let cols = parse_env_var(b"COLUMNS\0").unwrap_or(0);

    if lines == 0 || cols == 0 {
        let mut info = Winsize::default();
        let ret = ioctl(1, TIOCGWINSZ, &raw mut info);
        if ret == -1 {
            abort!("failet to call ioctl");
        }
        
        (info.ws_col as usize, info.ws_row as usize)
    } else {
        (cols, lines)
    }
}

pub fn format_timestamp(time: u64, format: Option<&str>) -> String {
    fn into_cstring(format: Option<&str>, defaut: CString) -> CString {
        if let Some(f) = format {
            CString::new(f).unwrap_or_else(|e| {
                warning!("Nul byte error in timestamp format: {e}");
                defaut
            })
        } else {
            defaut
        }
    }
    
    let mut info;
    let mut buf = [c_char::default(); 128 + 1];
    
    info = localtime((&raw const time).cast());
    if !info.is_null() {
        let c_format = into_cstring(format, c"%Y-%m-%d %H:%M:%S".into());
        strftime(buf.as_mut_ptr(), 128, c_format.as_ptr(), info);

        // SAFETY: Libc always returns correct cstring
        return unsafe { CStr::from_ptr(buf.as_ptr()) }
            .to_string_lossy()
            .into_owned()
    }
    
    info = gmtime((&raw const time).cast());
    if !info.is_null() {
        let c_format = into_cstring(format, c"%Y-%m-%d %H:%M:%S UTC".into());
        strftime(buf.as_mut_ptr(), 128, c_format.as_ptr(), info);
        
        // SAFETY: Libc always returns correct cstring
        return unsafe { CStr::from_ptr(buf.as_ptr()) }
            .to_string_lossy()
            .into_owned()
    }

    warning!("Failed to format timestamp");
    time.to_string()
}

#[cfg(test)]
mod tests {
    use crate::unix::env;

    #[test]
    fn terminal_size_test() {
        let (w, h) = env::terminal_size();
        println!("({w}, {h})");
    }
}