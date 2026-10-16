pub type ErrnoResult = Result<usize, Errno>;

#[repr(transparent)]
pub struct Errno(u16);

impl Errno {
    #[must_use]
    pub const fn new(val: u16) -> Self {
        Self(val)
    }

    /// # Errors
    ///
    /// Returns `Err(Errno)` if the value is such, otherwise `Ok(usize)`
    pub fn into_result(val: isize) -> ErrnoResult {
        if is_errno(val) {
            Err(Self::new(val as u16))
        } else {
            Ok(val as usize)
        }
    }
}

#[must_use]
pub fn is_errno(val: isize) -> bool {
    (-4095..0).contains(&val)
}
