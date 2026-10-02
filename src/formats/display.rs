use core::fmt::{
    self, 
    Debug, 
    Display, 
    Formatter
};

pub type ZeroPaddedTwo<T> = ZeroPadded<T, 2>;

pub struct ZeroPadded<T, const W: usize> {
    value: T
}

impl<T, const W: usize> ZeroPadded<T, W> {
    pub const fn new(value: T) -> Self {
        Self { value }
    }
}

impl<T: Display, const W: usize> Display for ZeroPadded<T, W> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:0width$}", self.value, width = W)
    }
}

impl<T: Display, const W: usize> Debug for ZeroPadded<T, W> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:0width$}", self.value, width = W)
    }
}