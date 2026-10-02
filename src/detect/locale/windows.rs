use core::ptr;

use alloc::string::String;

use crate::{
    str::SmolStr, 
    windows::{
        error::ErrorCode,
        link::GetLocaleInfoEx
    }
};

const LOCALE_NAME_MAX_LENGTH  : usize      = 85;
const LOCALE_NAME_USER_DEFAULT: *const u16 = ptr::null_mut();
const LOCALE_SNAME            : u32        = 0x005c;

pub fn get() -> SmolStr {
    let mut buf = [0u16; LOCALE_NAME_MAX_LENGTH];

    // SAFETY: Just a WinAPI function, the return value is checked
    let ret = unsafe {
        GetLocaleInfoEx(
            LOCALE_NAME_USER_DEFAULT, 
            LOCALE_SNAME, 
            buf.as_mut_ptr() , 
            LOCALE_NAME_MAX_LENGTH as i32
        )
    };
    if ret == 0 { ErrorCode::last().panic(); }

    let mut len = 0;
    while len < LOCALE_NAME_MAX_LENGTH && buf[len] != 0 {
        len += 1;
    }

    let string = String::from_utf16_lossy(&buf[..len]);
    SmolStr::from(string)
}