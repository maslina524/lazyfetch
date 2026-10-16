use crate::{io::write, syscall, table::Table};

pub fn exit(code: u32) -> ! {
    // SAFETY: We pass a valid nr,
    // all values are synchronized with https://syscalls.mebeim.net
    unsafe { syscall!(Table::Exit, code) };

    // The program should already terminate here
    let _ = write(2, b"Failed to terminate the program correctly!");
    loop {
        core::hint::spin_loop();
    }
}
