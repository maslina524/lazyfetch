use core::{ffi::CStr, ptr};

use crate::{
    linux::libc::setlocale,
    str::SmolStr
};

const LC_ALL: i32 = 0;

pub fn get() -> SmolStr {
    setlocale(LC_ALL, c"".as_ptr());
    let ptr = setlocale(LC_ALL, ptr::null());
    // SAFETY: Libs are guaranteed to store a valid cstr
    let c_str = unsafe { CStr::from_ptr(ptr) };

    SmolStr::from(c_str.to_string_lossy())
}