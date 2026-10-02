use core::fmt::Write;

use alloc::{
    boxed::Box, 
    string::String, 
    vec::Vec
};

#[derive(Clone, Copy)]
pub struct ConcatStr<const N: usize> {
    parts: [&'static str; N]
}

impl<const N: usize> ConcatStr<N> {
    pub const fn new(parts: [&'static str; N]) -> Self {
        Self { parts }
    }

    pub fn len(&self) -> usize {
        self.parts.iter().map(|s| s.len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.parts.iter().all(|s| s.is_empty())
    }

    pub fn chunks(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.parts.iter().copied()
    }

    pub fn bytes(&self) -> impl Iterator<Item = u8> + '_ {
        self.parts.iter().flat_map(|s| s.bytes())
    }

    pub fn as_boxed_str(&self) -> Box<str> {
        let total = self.len();
        let mut buf: Vec<u8> = Vec::with_capacity(total);
        for part in self.parts {
            buf.extend_from_slice(part.as_bytes());
        }
        debug_assert_eq!(buf.len(), total);
        // SAFETY: Concat &str = valid UTF-8
        // buf.len() == buf.capacity() == total
        // No extra allocations
        unsafe { String::from_utf8_unchecked(buf) }.into_boxed_str()
    }
}

impl<const N: usize> core::fmt::Display for ConcatStr<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for p in self.parts {
            if p.is_empty() {
                continue;
            }
            f.write_str(p)?;
        }
        Ok(())
    }
}

impl<const N: usize> core::fmt::Debug for ConcatStr<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_char('"')?;
        for p in self.parts {
            if p.is_empty() {
                continue;
            }
            for c in p.chars() {
                c.escape_debug().fmt(f)?;
            }
        }
        f.write_char('"')?;
        Ok(())
    }
}

impl<const N: usize> Default for ConcatStr<N> {
    fn default() -> Self {
        Self::new([""; N])
    }
}

impl From<&'static str> for ConcatStr<1> {
    fn from(value: &'static str) -> Self {
        Self { parts: [value] }
    }
}