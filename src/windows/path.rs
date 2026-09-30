use core::{
    ptr,
    slice
};

use alloc::{
    string::String,
    vec::Vec,
    borrow::ToOwned
};

use crate::{
    windows::{
        encoding::{wide, utf16le_to_utf8, Utf16Len},
        error,
        link::{GUID, SHGetKnownFolderPath, PathFileExistsW},
        env
    }
};

const FOLDERID_LOCALAPPDATA: GUID = GUID::from_u128(
    0xF1B32785_6FBA_4FCF_9D55_7B8E7F157091
);

#[repr(transparent)]
#[derive(Clone)]
pub struct Path {
    inner: String
}

impl Path {
    pub const fn new() -> Self {
        Self { inner: String::new() }
    }

    pub fn local() -> Self {
        if let Ok(s) = env::get_var("LOCALAPPDATA", None) {
            return Self::from(s);
        }

        let mut path_ptr = ptr::null_mut();

        // SAFETY: Completely safe
        let ret = unsafe {
            SHGetKnownFolderPath(
                &FOLDERID_LOCALAPPDATA, 
                0, 
                ptr::null_mut(), 
                &raw mut path_ptr
            )
        };
        assert!(ret == 0, "Failed to get the path to AppData/Local");

        let mut len = 0;
        // SAFETY: The string is located at the pointer and up to the null byte, 
        // everything is safe
        while unsafe { path_ptr.add(len).read() } != 0 {
            len += 1;
        }

        // SAFETY: Between `path_ptr..path_ptr + len` there is a string
        let slice = unsafe { slice::from_raw_parts(path_ptr, len) };
        let inner = utf16le_to_utf8(slice, Utf16Len::Len(len)).expect("UNREACHABLE");

        Self::from(inner)
    }

    pub fn lazyfetch() -> Self {
        Self::local().join("lazyfetch")
    }

    pub fn cache() -> Self {
        Self::lazyfetch().join("cache")
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self { inner: String::with_capacity(cap) }
    }

    fn clear(self) -> Self {
        let inner = self.inner
            .replace('\\', "/")
            .split('/')
            .filter(|x| !x.is_empty())
            .fold(String::with_capacity(self.inner.len() + 16), |mut acc, s| {
                if !acc.is_empty() {
                    acc.push('/');
                }
                acc.push_str(s);
                acc
            });

        Self { inner }
    }

    pub const fn as_str(&self) -> &String {
        &self.inner
    }

    pub fn as_wide_str(&self) -> error::Result<Vec<u16>> {
        wide(&self.inner)
    }

    pub fn parts(&self) -> Vec<&str> {
        self.inner.split('/').collect()
    }

    pub fn pop(&mut self) -> Option<String> {
        let mut parts = self.parts();
        if parts.len() <= 1 {
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
        let Ok(path_wide) = self.as_wide_str() else {
            return false;
        };
        // SAFETY: Completely safe
        let ret = unsafe {
            PathFileExistsW(path_wide.as_ptr())
        };
        ret == 1
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
        Self { inner: value.to_owned() }.clear()
    }
}

impl From<&str> for Path {
    fn from(value: &str) -> Self {
        Self { inner: value.to_owned() }.clear()
    }
}

impl From<Vec<String>> for Path {
    fn from(value: Vec<String>) -> Self {
        Self { inner: value.join("/") }.clear()
    }
}

impl From<Vec<&str>> for Path {
    fn from(value: Vec<&str>) -> Self {
        Self { inner: value.join("/") }.clear()
    }
}

impl From<&Self> for Path {
    fn from(value: &Self) -> Self {
        Self { inner: value.as_str().to_owned() }.clear()
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
    use crate::windows::path::Path;
  
    extern crate std;

    #[test]
    fn get_local_test() {
        let local = Path::local();
        let string = local.as_str();
        println!("{string}");
        assert!(string.ends_with("/AppData/Local"));
    }
}