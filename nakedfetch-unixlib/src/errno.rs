pub type UsizeErrnoResult = Result<usize, Errno>;
pub type ErrnoResult<T> = Result<T, Errno>;

#[repr(transparent)]
#[derive(Debug)]
pub struct Errno(u16);

impl Errno {
    pub const ENOMEM: Self = Self::new(12);

    #[must_use]
    pub const fn new(val: u16) -> Self {
        Self(val)
    }

    /// # Errors
    ///
    /// Returns `Err(Errno)` if the value is such, otherwise `Ok(usize)`
    pub fn into_result(val: isize) -> UsizeErrnoResult {
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
