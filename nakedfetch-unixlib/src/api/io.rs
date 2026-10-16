use crate::{
    errno::{Errno, UsizeErrnoResult},
    syscall,
    table::Table,
};

pub fn write(fd: isize, buf: &[u8]) -> UsizeErrnoResult {
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
        let ret = write(1, b"Hello World\n");
        assert!(ret.is_ok());
    }
}
