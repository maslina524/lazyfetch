use core::ffi::c_int;

use crate::unix::libc;

pub const fn stdout() -> isize {
    1
}

pub const fn stderr() -> isize {
    2
}

pub fn write(handle: isize, s: &[u8]) {
    libc::write(handle as c_int, s.as_ptr().cast(), s.len());
}

#[cfg(test)]
#[allow(clippy::print_with_newline, clippy::print_literal)]
mod tests {
    use crate::{
        unix::io::{stdout, write}
    };

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