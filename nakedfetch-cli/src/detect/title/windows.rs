use core::{
    ptr,
    slice
};

use alloc::{
    string::String,
    vec::Vec
};

use crate::{
    format,
    logo::LogoInfo,
    imp::env,
    imp::path::Path,
    imp::link::{
        ConvertSidToStringSidW, GetComputerNameW, GetCurrentDirectoryW, GetCurrentProcess,
        GetCurrentProcessId, GetTokenInformation, GetUserNameExW, GetUserNameW, OpenProcessToken,
        LocalFree, CloseHandle, PSID
    },
    detect::title::TitleInfo
};

const UNLEN      : usize            = 256;
const PCNAME_LEN : usize            = 15;
const TOKEN_QUERY: u32              = 8;

#[repr(C)]
#[derive(Clone)]
struct SID_AND_ATTRIBUTES {
    pub sid: PSID,
    pub attributes: u32,
}

#[repr(C)]
#[derive(Clone)]
struct TOKEN_USER {
    pub user: SID_AND_ATTRIBUTES,
}

impl TitleInfo {
    pub fn new() -> Self {
        let user_name = Self::user_name();
        let user_name_colored = Self::colored(&user_name);
        let host_name = Self::host_name();
        let host_name_colored = Self::colored(&host_name);
        let home_dir = Self::home_dir(&user_name);
        let exe_path = Self::exe_path();
        let user_shell = Self::user_shell();
        let full_user_name = Self::full_user_name();
        let user_id = Self::user_id();
        let pid = Self::pid();
        let cwd = Self::cwd();

        Self { 
            user_name, 
            host_name, 
            home_dir, 
            exe_path, 
            user_shell, 
            user_name_colored,
            host_name_colored, 
            full_user_name, 
            user_id, 
            pid, cwd 
        }
    }

    fn user_name() -> String {
        let mut buf = [0u16; UNLEN + 1];
        let mut size = (UNLEN + 1) as u32;
        
        // SAFETY: Completely safe
        let ret = unsafe {
            GetUserNameW(
                (&raw mut buf).cast(),
                &raw mut size
            )
        };
        if ret == 0 {
            return String::new();
        }

        String::from_utf16_lossy(&buf).rsplit('\0').collect()
    }

    fn colored(s: &str) -> String {
        format!(
            "\x1b[1;{}m{}\x1b[0m",
            LogoInfo::get().unwrap().color_title,
            s
        )
    }

    fn host_name() -> String {
        let mut buf = [0u16; PCNAME_LEN + 1];
        let mut size = (UNLEN + 1) as u32;
        
        // SAFETY: Completely safe
        let ret = unsafe {
            GetComputerNameW(
                (&raw mut buf).cast(),
                &raw mut size,
            )
        };
        if ret == 0 {
            return String::new();
        }

        String::from_utf16_lossy(&buf).to_lowercase().rsplit('\0').collect()
    }

    fn full_user_name() -> String {
        let mut buf = [0u16; UNLEN + 1];
        let mut size = (UNLEN + 1) as u32;
        
        // SAFETY: Completely safe
        let ret = unsafe {
            GetUserNameExW(
                3,
                (&raw mut buf).cast(),
                &raw mut size,
            )
        };
        if !ret {
            return String::new();
        }

        String::from_utf16_lossy(&buf).rsplit('\0').collect()
    }

    pub fn home_dir(user_name: &str) -> Path {
        Path::from("C:/Users").join(user_name)
    }

    pub fn exe_path() -> Path {
        let args = env::args();
        Path::from(args[0].clone())
    }

    pub const fn user_shell() -> Path {
        Path::new() // No impl in windows
    }

    pub fn cwd() -> Path {
        let mut buf = [0u16; 1024];
        let size = 1024;
        
        // SAFETY: Completely safe
        let ret = unsafe {
            GetCurrentDirectoryW(
                size,
                (&raw mut buf).cast(),
            )
        };
        if ret == 0 {
            return Path::new();
        }

        let string: String = String::from_utf16_lossy(&buf).rsplit('\0').collect();
        Path::from(string)
    }

    pub fn pid() -> u32 {
        // SAFETY: Completely safe
        unsafe {
            GetCurrentProcessId()
        }
    }

    pub fn user_id() -> String {
        // SAFETY: Completely safe
        let cur_proc = unsafe { GetCurrentProcess() };
        let mut token  = ptr::null_mut();

        // SAFETY: Completely safe
        let ret = unsafe {
            OpenProcessToken(
                cur_proc, 
                TOKEN_QUERY, 
                &raw mut token
            )
        };
        if ret == 0 {
            return String::new();
        }

        let mut size = 0;
        // SAFETY: Completely safe
        unsafe {
            GetTokenInformation(
                token, 
                1, 
                ptr::null_mut(), 
                0, 
                &raw mut size
            ) 
        };
        if size == 0 {
            // SAFETY: Completely safe
            unsafe { CloseHandle(token) };
            return String::new();
        }

        let mut buffer: Vec<TOKEN_USER> = Vec::with_capacity(size as usize);
        // SAFETY: Completely safe
        let ret = unsafe {
            GetTokenInformation(
                token,
                1,
                buffer.as_mut_ptr().cast(),
                size,
                &raw mut size,
            )
        };
        if ret == 0 {
            // SAFETY: Completely safe
            unsafe { CloseHandle(token) };
            return String::new();
        }

        let ptoken_user = buffer.as_ptr().cast::<TOKEN_USER>();
        // SAFETY: WinAPI returns a valid pointer to `TOKEN_USER`
        let token_user = unsafe {
            (*ptoken_user).clone()
        };
        let sid = token_user.user.sid;

        let mut sid_str = ptr::null_mut();
        // SAFETY: Completely safe
        let ret = unsafe {
            ConvertSidToStringSidW(
                sid,
                &raw mut sid_str
            )
        };
        if ret == 0 {
            // SAFETY: Completely safe
            unsafe { CloseHandle(token) };
            return String::new();
        }

        let result = if sid_str.is_null() {
            String::new()
        } else {
            let mut len = 0;
            // SAFETY: Up to the `\0`, this is a string allocated by WinAPI
            while unsafe { *sid_str.add(len) } != 0 {
                len += 1;
            }

            // SAFETY: Creating a string from memory allocated by WinAPI
            let slice = unsafe {
                slice::from_raw_parts(sid_str, len)
            };

            String::from_utf16_lossy(slice)
        };

        // SAFETY: Completely safe
        unsafe {
            LocalFree(sid_str.cast());
            CloseHandle(token);
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use crate::detect::title::TitleInfo;

    extern crate std;

    #[test]
    fn cwd_test() {
        let cwd = TitleInfo::cwd();
        println!("{cwd}");
    }

    #[test]
    fn host_test() {
        let cwd = TitleInfo::host_name();
        println!("{cwd}");
    }

    #[test]
    fn user_id_test() {
        let user_id = TitleInfo::user_id();
        println!("{user_id}");
    }
}