use core::ops::Deref;

use alloc::{
    borrow::Cow, 
    string::String, 
    sync::Arc
};

const INLINE_CAP: usize = 30;

#[derive(Default, Clone)]
pub enum Repr {
    #[default]
    Empty,
    Inline { 
        len: u8, data: [u8; INLINE_CAP] 
    },
    Heap(Arc<str>),
    Static(&'static str)
}

/// [`SmolStr`] only stores the string, does not modify it
#[derive(Default, Clone)]
pub struct SmolStr(Repr);

impl SmolStr {
    pub const fn empty() -> Self {
        Self(Repr::Empty)
    }

    pub const fn from_static(s: &'static str) -> Self {
        Self(Repr::Static(s))
    }

    pub const fn from_slice(slice: [u8; INLINE_CAP], len: usize) -> Self {
        Self(Repr::Inline { len: len as u8, data: slice })
    }

    pub fn as_str(&self) -> &str {
        match &self.0 {
            Repr::Empty => "",
            Repr::Heap(a) => a,
            // SAFETY: `SmolStr` is created only from `str`, utf8 is always valid.
            Repr::Inline { len, data } => unsafe { 
                str::from_utf8_unchecked(&data[..*len as usize]) 
            },
            Repr::Static(s) => s
        }
    }
}

impl Deref for SmolStr {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

#[inline]
fn str_to_arr(s: &str) -> [u8; INLINE_CAP] {
    let mut out = [0u8; INLINE_CAP];
    for (dst, c) in out.iter_mut().zip(s.chars()) {
        *dst = c as u8;
    }
    out
}

macro_rules! impl_from_string {
    ($($typ:ty),+ $(,)?) => {$(
        impl From<$typ> for SmolStr {
            fn from(value: $typ) -> Self {
                let repr = if value.len() > INLINE_CAP {
                    Repr::Heap(Arc::from(value))
                } else {
                    Repr::Inline { 
                        len: value.len() as u8, 
                        data: str_to_arr(&value) 
                    }
                };

                Self(repr)
            }
        }
    )+};
}

impl_from_string!(
    String,
    &str,
    Cow<'static, str>
);

impl core::fmt::Display for SmolStr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl core::fmt::Debug for SmolStr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.as_str())
    }
}