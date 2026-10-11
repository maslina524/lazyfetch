use core::fmt::{self, Debug, Display, Formatter};

use crate::lua::{AsLua, LuaType};

pub type ZeroPaddedTwo<T> = ZeroPadded<T, 2>;

pub trait NumTrait: Display + Copy {}

macro_rules! impl_numtrait {
    ($($item:ident),+ $(,)?) => {$(
        impl NumTrait for $item {}
    )+};
}

impl_numtrait!(
    i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, isize, usize
);

pub struct ZeroPadded<N: NumTrait, const W: usize> {
    value: N,
}

impl<N: NumTrait, const W: usize> ZeroPadded<N, W> {
    pub const fn new(value: N) -> Self {
        Self { value }
    }
}

impl<N: NumTrait, const W: usize> Display for ZeroPadded<N, W> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:0width$}", self.value, width = W)
    }
}

impl<N: NumTrait, const W: usize> Debug for ZeroPadded<N, W> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:0width$}", self.value, width = W)
    }
}

impl<N: NumTrait + Into<f64>, const W: usize> AsLua for ZeroPadded<N, W> {
    fn as_lua(&self) -> LuaType {
        LuaType::Number(self.value.into())
    }
    const LUA_TYPE: &'static str = "number";
}
