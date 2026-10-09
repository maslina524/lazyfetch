use core::ffi::CStr;

use alloc::{borrow::ToOwned, ffi::CString, string::String, vec::Vec};

use crate::{
    abort,
    unix::libc::{access, getenv},
};

#[repr(transparent)]
#[derive(Clone)]
pub struct Path {
    inner: String,
}

impl Path {
    pub const fn new() -> Self {
        Self {
            inner: String::new(),
        }
    }

    // ONLY FOR LINUX
    fn home() -> Self {
        let raw_name = c"HOME";
        let ptr = getenv(raw_name.as_ptr());
        if ptr.is_null() {
            abort!("Failed to get HOME");
        }
        // SAFETY: Libs are guaranteed to store a valid cstr
        let c_str = unsafe { CStr::from_ptr(ptr) };
        let string = c_str.to_string_lossy().into_owned();
        Self::from(string)
    }

    pub fn local() -> Self {
        Self::home().join(".local/share")
    }

    pub fn lazyfetch() -> Self {
        Self::local().join("lazyfetch")
    }

    pub fn cache() -> Self {
        Self::home().join("cache/lazyfetch")
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            inner: String::with_capacity(cap),
        }
    }

    fn clear(mut self) -> Self {
        let cap = self.inner.len();
        let is_absolute = self.inner.starts_with('/');
        self.inner = self.inner.replace('\\', "/");
        let parts: Vec<&str> = self.inner.split('/').filter(|x| !x.is_empty()).collect();
        let mut new_inner = String::with_capacity(cap + 8);

        if is_absolute {
            new_inner.push('/');
        }

        for (i, part) in parts.iter().enumerate() {
            if i > 0 {
                new_inner.push('/');
            }
            new_inner.push_str(part);
        }
        self.inner = new_inner;
        self
    }

    pub const fn as_str(&self) -> &String {
        &self.inner
    }

    // ONLY FOR LINUX
    pub fn as_c_str(&self) -> CString {
        CString::new(self.inner.clone()).unwrap()
    }

    pub fn parts(&self) -> Vec<&str> {
        self.inner.split('/').collect()
    }

    pub fn pop(&mut self) -> Option<String> {
        let mut parts = self.parts();
        if parts.is_empty() {
            return None;
        }
        let popped = parts.pop().map(String::from);
        self.inner = parts.join("/");
        popped
    }

    pub fn last(&'_ self) -> Option<&'_ str> {
        let parts = self.parts();
        parts.last().copied()
    }

    pub fn join(&self, path: impl Into<Self>) -> Self {
        let mut parts = self.parts();
        let path = path.into();
        let path_parts = path.parts();
        parts.extend(path_parts);
        Self::from(parts)
    }

    pub fn exists(&self) -> bool {
        let c_str = self.as_c_str();
        // SAFETY: Completely safe
        access(c_str.as_ptr(), 0) == 0
    }

    pub fn parent(&self) -> Option<Self> {
        let mut clone = self.clone();
        clone.pop()?;
        Some(clone)
    }

    pub fn into_inner(self) -> String {
        self.inner
    }
}

impl From<String> for Path {
    fn from(value: String) -> Self {
        Self { inner: value }.clear()
    }
}

impl From<&String> for Path {
    fn from(value: &String) -> Self {
        Self {
            inner: value.to_owned(),
        }
        .clear()
    }
}

impl From<&str> for Path {
    fn from(value: &str) -> Self {
        Self {
            inner: value.to_owned(),
        }
        .clear()
    }
}

impl From<Vec<String>> for Path {
    fn from(value: Vec<String>) -> Self {
        Self {
            inner: value.join("/"),
        }
        .clear()
    }
}

impl From<Vec<&str>> for Path {
    fn from(value: Vec<&str>) -> Self {
        Self {
            inner: value.join("/"),
        }
        .clear()
    }
}

impl From<&Self> for Path {
    fn from(value: &Self) -> Self {
        Self {
            inner: value.as_str().to_owned(),
        }
        .clear()
    }
}

impl core::fmt::Display for Path {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl core::fmt::Debug for Path {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "\"{}\"", self.inner)
    }
}

impl Default for Path {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::unix::path::Path;

    #[test]
    fn get_local_test() {
        let local = Path::local();
        let string = local.as_str();
        println!("{string}");
        assert!(string.ends_with(".local/share"));
    }
}
