use core::ops::Deref;

use alloc::sync::Arc;

const INLINE_CAP: usize = 30;

pub enum Repr {
    Inline { 
        len: u8, data: [u8; INLINE_CAP] 
    },
    Heap(Arc<str>),
    Static(&'static str)
}

/// [`SmolStr`] only stores the string, does not modify it
pub struct SmolStr(Repr);

impl SmolStr {
    pub const fn from_static(s: &'static str) -> Self {
        Self(Repr::Static(s))
    }

    pub fn as_str(&self) -> &str {
        match &self.0 {
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

impl From<&str> for SmolStr {
    fn from(value: &str) -> Self {
        let repr = if value.len() > INLINE_CAP {
            Repr::Heap(Arc::from(value))
        } else {
            Repr::Inline { 
                len: value.len() as u8, 
                data: str_to_arr(value) 
            }
        };

        Self(repr)
    }
}