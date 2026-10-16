#[repr(usize)]
pub enum Table {
    Write = 1,
    Exit = 60,
    Mmap = 9,
    Munmap = 11,
}
