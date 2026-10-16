#[repr(usize)]
pub enum Table {
    Write = 64,
    Exit = 93,
    Mmap = 222,
    Munmap = 215,
}
