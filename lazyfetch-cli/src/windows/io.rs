use core::{
    ffi::c_void,
    ptr,
    sync::atomic::{AtomicBool, Ordering},
};

use crate::{
    sync::OnceLock,
    windows::error::ErrorCode,
    windows::link::{GetStdHandle, SetConsoleOutputCP, WriteFile},
};

const STDOUT: u32 = u32::MAX - 10;
const STDERR: u32 = u32::MAX - 11;
const CP_UTF8: u32 = 65001;

static STDOUT_HANDLE: OnceLock<isize> = OnceLock::new();
static STDERR_HANDLE: OnceLock<isize> = OnceLock::new();
static IS_UTF8: AtomicBool = AtomicBool::new(false);

pub fn stdout() -> isize {
    *STDOUT_HANDLE.get_or_init(|| {
        // SAFETY: Completely safe
        (unsafe { GetStdHandle(STDOUT) }) as isize
    })
}

pub fn stderr() -> isize {
    *STDERR_HANDLE.get_or_init(|| {
        // SAFETY: Completely safe
        (unsafe { GetStdHandle(STDERR) }) as isize
    })
}

pub fn write(handle: isize, s: &[u8]) {
    if IS_UTF8
        .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
        .is_ok()
    {
        // SAFETY: The function always receives a valid constant from the documentation,
        // and possible errors are checked
        let ret = unsafe { SetConsoleOutputCP(CP_UTF8) };
        if ret == 0 {
            ErrorCode::last().panic();
        }
    }
    let mut written = 0;
    let len = s.len() as u32;

    // SAFETY: The handle is always correct,
    // errors are checked, the function is safe
    let ret = unsafe {
        WriteFile(
            handle as *mut c_void,
            s.as_ptr(),
            len,
            &raw mut written,
            ptr::null_mut(),
        )
    };
    if ret == 0 || written != len {
        ErrorCode::last().panic();
    }
}

#[cfg(test)]
#[allow(clippy::print_with_newline, clippy::print_literal)]
mod tests {
    use crate::windows::io::{stdout, write};

    #[test]
    fn write_test() {
        write(stdout(), b"Hello World!\n");
        write(stdout(), "Привет мир!\n".as_bytes());
        write(stdout(), "👋👋👋\n".as_bytes());
    }

    #[test]
    fn print_test() {
        print!("Hello {}\n", "World");
        print!("2 + 8 = {}", 2 + 8);
        print!("\n");
    }

    #[test]
    fn println_test() {
        println!("Hello {}", "World");
        println!("2 + 8 = {}", 2 + 8);
    }

    #[test]
    fn eprints_test() {
        eprintln!("Hello {}", "World");
        eprint!("42 * 55 = {}", 42 * 55);
    }
}
