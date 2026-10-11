use core::{error::Error, fmt::Display};

use alloc::{
    string::{FromUtf16Error, String},
    vec::Vec,
};

use crate::windows::error::ErrorCode;

const CP_UTF8: u32 = 65001;

#[derive(Clone, Copy)]
pub enum Utf16Len {
    NullTerminated,
    Len(usize),
}

#[derive(Debug)]
pub enum Utf16ToUtf8 {
    FromUtf16Error(FromUtf16Error),
    NullNotFound,
}

impl Display for Utf16ToUtf8 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::FromUtf16Error(e) => write!(f, "FromUtf16Error: {e}"),
            Self::NullNotFound => write!(f, "Null byte in string not found"),
        }
    }
}

impl From<FromUtf16Error> for Utf16ToUtf8 {
    fn from(val: FromUtf16Error) -> Self {
        Self::FromUtf16Error(val)
    }
}

impl Error for Utf16ToUtf8 {}

#[derive(Debug)]
pub enum EncodeError {
    Utf16ToUtf8(Utf16ToUtf8),
    ErrorCode(ErrorCode),
}

impl Display for EncodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Utf16ToUtf8(e) => write!(f, "{e}"),
            Self::ErrorCode(e) => write!(f, "Error Code: {e}"),
        }
    }
}

impl From<Utf16ToUtf8> for EncodeError {
    fn from(val: Utf16ToUtf8) -> Self {
        Self::Utf16ToUtf8(val)
    }
}

impl From<ErrorCode> for EncodeError {
    fn from(val: ErrorCode) -> Self {
        Self::ErrorCode(val)
    }
}

impl Error for EncodeError {}

pub type Result<T> = core::result::Result<T, EncodeError>;

const fn find_null(buf: &[u16]) -> Option<usize> {
    let mut len = 0;
    loop {
        if len >= buf.len() {
            return None;
        }
        if buf[len] == 0 {
            return Some(len);
        }
        len += 1;
    }
}

pub fn utf16le_to_utf8(buf: &[u16], len: Utf16Len) -> core::result::Result<String, Utf16ToUtf8> {
    if matches!(len, Utf16Len::Len(0)) {
        return Ok(String::new());
    }

    let len = match len {
        Utf16Len::Len(l) => l,
        Utf16Len::NullTerminated => find_null(buf).ok_or(Utf16ToUtf8::NullNotFound)?,
    };

    let string = String::from_utf16(&buf[..len])?;
    Ok(string)
}

pub fn wide(s: &str) -> Vec<u16> {
    let mut buf = Vec::with_capacity(s.len() * 2 + 4);
    for u in s.encode_utf16() {
        buf.push(u);
    }
    buf.push(0);
    buf
}

pub fn wide_without_alloc(s: &str, buf: &mut [u16]) -> usize {
    let mut i = 0;
    for u in s.encode_utf16() {
        if i >= buf.len().saturating_sub(1) {
            break;
        }
        buf[i] = u;
        i += 1;
    }
    i
}
