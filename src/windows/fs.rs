use core::{
    ffi::c_void,
    ptr
};

use alloc::{
    string::{String, FromUtf8Error},
    vec::Vec,
    vec
};

use crate::{
    windows::error::{self, ErrorCode},
    windows::path::Path,
    windows::link::{
        CreateFileW, HANDLE, WriteFile, ReadFile,
        GetFileSizeEx, CloseHandle, CreateDirectoryW
    }
};

const FILE_SHARE_READ      : u32         = 0x0001;
const FILE_ATTRIBUTE_NORMAL: u32         = 0x0080;
const INVALID_HANDLE       : *mut c_void = (-1isize).cast_unsigned() as *mut c_void;
const CREATE_NEW           : u32         = 0x0001;
const CREATE_ALWAYS        : u32         = 0x0002;
const OPEN_EXISTING        : u32         = 0x0003;

type FileHandle = HANDLE;

#[repr(u32)]
pub enum Access {
    Read      = 0x8000_0000,
    Write     = 0x4000_0000,
    ReadWrite = 0x8000_0000 | 0x4000_0000,
    All       = 0x1000_0000
}

#[derive(Debug)]
pub enum ReadError {
    Utf8(FromUtf8Error),
    Code(ErrorCode),
}

impl From<FromUtf8Error> for ReadError {
    fn from(err: FromUtf8Error) -> Self {
        Self::Utf8(err)
    }
}

impl From<ErrorCode> for ReadError {
    fn from(err: ErrorCode) -> Self {
        Self::Code(err)
    }
}

impl core::fmt::Display for ReadError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Utf8(s) => write!(f, "Utf8Error: {s}"),
            Self::Code(s) => write!(f, "ErrorCode: {s}"),
        }
    }
}

impl core::error::Error for ReadError {}

pub struct File(FileHandle);

impl File {
    pub fn open(path: impl Into<Path>, access: Access) -> error::Result<Self> {
        Self::create_with_cd(path, access, OPEN_EXISTING)
    }

    pub fn create_new(path: impl Into<Path>, access: Access) -> error::Result<Self> {
        Self::create_with_cd(path, access, CREATE_NEW)
    }

    pub fn create_always(path: impl Into<Path>, access: Access) -> error::Result<Self> {
        Self::create_with_cd(path, access, CREATE_ALWAYS)
    }

    pub const fn as_handle(&self) -> FileHandle {
        self.0
    }

    fn create_with_cd(path: impl Into<Path>, access: Access, cd: u32) -> error::Result<Self> {
        let path = path.into();
        let path_wide = path.as_wide_str();

        // SAFETY: Parameters are fully correct, return value is checked
        let handle = unsafe {
            CreateFileW(
                path_wide.as_ptr(), 
                access as u32, 
                FILE_SHARE_READ, 
                ptr::null(), 
                cd, 
                FILE_ATTRIBUTE_NORMAL, 
                ptr::null_mut()
            )
        };
        if handle == INVALID_HANDLE {
            return Err(ErrorCode::last());
        }
        
        Ok(Self(handle))
    }

    pub fn write(&self, buf: &[u8]) -> error::Result<()> {
        let len = buf.len() as u32;
        let mut written = 0;

        // SAFETY: The handle is always correct, 
        // errors are checked, the function is safe
        let ret = unsafe {
            WriteFile(
                self.0, 
                buf.as_ptr(), 
                len, 
                &raw mut written, 
                ptr::null_mut()
            )
        };
        if ret == 0 || written != len {
            return Err(ErrorCode::last());
        }
        
        Ok(())
    }

    pub fn read(&self) -> error::Result<Vec<u8>> {
        let mut size = 0;
        // SAFETY: Completely safe
        let ret = unsafe {
            GetFileSizeEx(self.0, &raw mut size)
        };
        if ret == 0 {
            return Err(ErrorCode::last());
        }

        let mut buf = vec![0u8; size as usize];
        let mut readed = 0;

        // SAFETY: The handle is always correct, 
        // errors are checked, the function is safe
        unsafe {
            ReadFile(
                self.0,
                buf.as_mut_ptr(), 
                size as u32, 
                &raw mut readed, 
                ptr::null_mut()
            )
        };

        Ok(buf)
    }
}

impl Drop for File {
    fn drop(&mut self) {
        // SAFETY: Completely safe
        unsafe { CloseHandle(self.0) };
    }
}

pub fn read(path: impl Into<Path>) -> error::Result<Vec<u8>> {
    let handler = File::open(path, Access::Read)?;
    let bytes = handler.read()?;
    Ok(bytes)
}

pub fn read_to_string(path: impl Into<Path>) -> Result<String, ReadError> {
    let handler = File::open(path, Access::Read)?;
    let bytes = handler.read()?;
    let string = String::from_utf8(bytes)?;
    Ok(string)
}

pub fn create_dir(path: impl Into<Path>) -> error::Result<()> {
    let path = path.into();
    let path_wide = path.as_wide_str();

    // SAFETY: Parameters are fully correct, return value is checked
    let ret = unsafe {
        CreateDirectoryW(
            path_wide.as_ptr(),
            ptr::null()
        )
    };
    if ret == 0 {
        return Err(ErrorCode::last());
    }

    Ok(())
}

pub fn create_dirs(path: impl Into<Path>) -> error::Result<()> {
    let path = path.into();
    if !path.exists() {
        if let Some(parent) = path.parent() {
            create_dirs(parent)?;
        }
        create_dir(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use alloc::string::String;

    use crate::windows::fs::{self, Access, File};
  
    extern crate std;

    #[test]
    fn write_file_test() {
        let file = File::open("test.txt", Access::Write).unwrap();
        let _ = file.write(b"Hello World!");
    }

    #[test]
    fn read_file_test() {
        let file = File::open("README.md", Access::Read).unwrap();
        let buf = file.read().unwrap();
        let string = String::from_utf8(buf).unwrap();

        assert!(string.starts_with("# Lazyfetch"));
    }

    #[test]
    fn read_to_string_test() {
        let string = fs::read_to_string("README.md").unwrap();
        assert!(string.starts_with("# Lazyfetch"));
    }

    #[test]
    fn create_dirs_test() {
        fs::create_dirs("a/b/c/d").expect("ErrorCode");
    }
}