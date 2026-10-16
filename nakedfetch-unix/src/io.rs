use crate::{
    errno::{Errno, ErrnoResult},
    syscall,
    table::Table,
};

pub fn write(fd: u32, buf: &[u8]) -> ErrnoResult {
    // SAFETY: We pass a valid nr,
    // all values are synchronized with https://syscalls.mebeim.net
    let ret = unsafe { syscall!(Table::Write, fd, buf.as_ptr(), buf.len()) };
    Errno::into_result(ret)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_test() {
        let ret = write(0, b"Hello World\n");
        assert!(ret.is_ok());
    }
}