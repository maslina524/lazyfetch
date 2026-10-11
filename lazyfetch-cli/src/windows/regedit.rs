use core::{
    ffi::c_void,
    ptr,
    sync::atomic::{AtomicPtr, Ordering},
};

use alloc::{string::String, vec, vec::Vec};

use crate::{
    windows::encoding::wide,
    windows::error::{self, ErrorCode},
    windows::link::{RegCloseKey, RegCreateKeyExW, RegOpenKeyExW, RegQueryValueExW},
};

pub type Handle = AtomicPtr<c_void>;

#[repr(u32)]
pub enum Hkey {
    ClassesRoot = 0x8000_0000,
    CurrentUser = 0x8000_0001,
    LocalMachine = 0x8000_0002,
    Users = 0x8000_0003,
    CurrentConfig = 0x8000_0005,
}

#[repr(u32)]
pub enum Access {
    Read = 0x20019,
    Set = 0x0002,
    Write = 0x20006,
    All = 0xF003F,
}

#[derive(Debug, Clone)]
pub enum RegValue {
    Binary(Vec<u8>),
    Dword(u32),
    DwordLE(u32),
    DwordBE(u32),
    ExpandSz(String),
    Link(String),
    MultiSz(Vec<String>),
    None,
    Qword(u64),
    QwordLE(u64),
    Sz(String),
}

impl RegValue {
    pub const fn as_u32(&self) -> Option<u32> {
        match self {
            Self::Dword(v) | Self::DwordLE(v) => Some(*v),
            Self::DwordBE(v) => Some(v.to_be()),
            _ => None,
        }
    }

    pub const fn as_u64(&self) -> Option<u64> {
        match self {
            Self::Qword(v) | Self::QwordLE(v) => Some(*v),
            _ => None,
        }
    }

    pub const fn as_string(&self) -> Option<&str> {
        match self {
            Self::Sz(s) | Self::ExpandSz(s) | Self::Link(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub const fn as_multi_string(&self) -> Option<&[String]> {
        match self {
            Self::MultiSz(v) => Some(v.as_slice()),
            _ => None,
        }
    }

    pub const fn as_binary(&self) -> Option<&[u8]> {
        match self {
            Self::Binary(v) => Some(v.as_slice()),
            _ => None,
        }
    }

    pub const fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

pub struct Regedit(Handle);

// SAFETY: Completely safe
unsafe impl Sync for Regedit {}

impl Regedit {
    pub fn create(root: Hkey, subkey: &str, access: Access) -> error::Result<Self> {
        let mut handle = ptr::null_mut();
        let wide = wide(subkey);

        // SAFETY: All parameters have been verified
        // against the documentation, safe
        let ret = unsafe {
            RegCreateKeyExW(
                root as u32 as *mut c_void,
                wide.as_ptr(),
                0,
                ptr::null(),
                0,
                access as u32,
                ptr::null(),
                &raw mut handle,
                ptr::null_mut(),
            )
        };
        if ret != 0 {
            return Err(ErrorCode::last());
        }

        Ok(Self(AtomicPtr::new(handle)))
    }

    pub fn open(root: Hkey, subkey: &str, access: Access) -> error::Result<Self> {
        let mut handle = ptr::null_mut();
        let wide = wide(subkey);

        // SAFETY: All parameters have been verified
        // against the documentation, safe
        let ret = unsafe {
            RegOpenKeyExW(
                root as u32 as *mut c_void,
                wide.as_ptr(),
                0,
                access as u32,
                &raw mut handle,
            )
        };
        if ret != 0 {
            return Err(ErrorCode::last());
        }

        Ok(Self(AtomicPtr::new(handle)))
    }

    pub const fn from_handle(handle: Handle) -> Self {
        Self(handle)
    }

    pub fn read(&self, key: &str) -> error::Result<RegValue> {
        let mut size = 0;
        let mut typ = 0;
        let wide = wide(key);

        // SAFETY: Getting the buffer size and type, safe
        let ret = unsafe {
            RegQueryValueExW(
                self.0.load(Ordering::Acquire),
                wide.as_ptr(),
                ptr::null(),
                &raw mut typ,
                ptr::null_mut(),
                &raw mut size,
            )
        };
        if ret != 0 {
            return Err(ErrorCode::last());
        }

        let mut buf = vec![0u8; size as usize];

        // SAFETY: Completely safe
        unsafe {
            RegQueryValueExW(
                self.0.load(Ordering::Acquire),
                wide.as_ptr(),
                ptr::null(),
                &raw mut typ,
                buf.as_mut_ptr(),
                &raw mut size,
            )
        };

        match typ {
            0 => Ok(RegValue::None),
            1 => {
                // SAFETY: Using memory specifically allocated for the string
                let u16_slice = unsafe {
                    let len = buf.len() / 2;
                    core::slice::from_raw_parts(buf.as_ptr().cast(), len)
                };
                let chars = u16_slice
                    .iter()
                    .take_while(|&&c| c != 0)
                    .copied()
                    .collect::<Vec<u16>>();
                let s = String::from_utf16_lossy(&chars);
                Ok(RegValue::Sz(s))
            }
            2 => {
                // SAFETY: Using memory specifically allocated for the string
                let u16_slice = unsafe {
                    let len = buf.len() / 2;
                    core::slice::from_raw_parts(buf.as_ptr().cast(), len)
                };
                let chars = u16_slice
                    .iter()
                    .take_while(|&&c| c != 0)
                    .copied()
                    .collect::<Vec<u16>>();
                let s = String::from_utf16_lossy(&chars);
                Ok(RegValue::ExpandSz(s))
            }
            4 => {
                if buf.len() != 4 {
                    return Err(ErrorCode::new(13)); // ERROR_INVALID_DATA
                }
                let val = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
                Ok(RegValue::Dword(val))
            }
            5 => {
                if buf.len() != 4 {
                    return Err(ErrorCode::new(13));
                }
                let val = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
                Ok(RegValue::DwordBE(val))
            }
            6 => {
                // SAFETY: Using memory specifically allocated for the string
                let u16_slice = unsafe {
                    let len = buf.len() / 2;
                    core::slice::from_raw_parts(buf.as_ptr().cast(), len)
                };
                let chars = u16_slice
                    .iter()
                    .take_while(|&&c| c != 0)
                    .copied()
                    .collect::<Vec<u16>>();
                let s = String::from_utf16_lossy(&chars);
                Ok(RegValue::Link(s))
            }
            7 => {
                // SAFETY: Using memory specifically allocated for the string
                let u16_slice = unsafe {
                    let len = buf.len() / 2;
                    core::slice::from_raw_parts(buf.as_ptr().cast(), len)
                };

                let u16_slice_len = u16_slice.len();
                let mut strings = Vec::with_capacity(u16_slice_len);
                let mut start = 0;
                while start < u16_slice_len {
                    let end = u16_slice[start..]
                        .iter()
                        .position(|&c| c == 0)
                        .map_or(u16_slice.len(), |pos| start + pos);
                    if start == end {
                        break;
                    }
                    let s = String::from_utf16_lossy(&u16_slice[start..end]);
                    strings.push(s);
                    start = end + 1;
                    if start < u16_slice.len() && u16_slice[start] == 0 {
                        break;
                    }
                }
                Ok(RegValue::MultiSz(strings))
            }
            11 => {
                if buf.len() != 8 {
                    return Err(ErrorCode::new(13));
                }
                let val = u64::from_le_bytes([
                    buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
                ]);
                Ok(RegValue::Qword(val))
            }
            _ => Ok(RegValue::Binary(buf)),
        }
    }

    fn close(&mut self) -> error::Result<()> {
        // SAFETY: The handle is always valid,
        // the function is only used in `Drop`
        let ret = unsafe { RegCloseKey(self.0.load(Ordering::Acquire)) };
        if ret != 0 {
            return Err(ErrorCode::last());
        }
        Ok(())
    }
}

impl Drop for Regedit {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        modules::{Module, cpu::Cpu},
        windows::regedit::{Access, Hkey, Regedit},
    };

    extern crate std;

    #[test]
    fn open_test() {
        let handle = Regedit::open(
            Hkey::LocalMachine,
            "HARDWARE\\DESCRIPTION\\System\\CentralProcessor\\0",
            Access::Read,
        )
        .unwrap();
        let key = handle.read("VendorIdentifier").unwrap();

        let reg_vendor = key.as_string().unwrap();
        let cpuid_vendor = Cpu::new().vendor;

        assert_eq!(reg_vendor, cpuid_vendor);
    }
}
