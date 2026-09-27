use core::{
    ffi::{CStr, c_int, c_char},
    ptr
};

use alloc::{
    string::{FromUtf8Error, String},
    vec::Vec,
    vec
};

use crate::{
    linux::libc::{
        FileHandle, fopen, fread, fclose, fwrite, rewind, mkdir,
        Dir, opendir, readdir, readlink, ferror
    },
    linux::error::{self, ErrorCode},
    linux::path::Path
};

use crate::linux::libc::{open, close, getdents64, LinuxDirent64};
const O_RDONLY: c_int = 0;

#[cfg(target_arch = "aarch64")]
const O_DIRECTORY: c_int = 0x4000;
#[cfg(not(target_arch = "aarch64"))]
const O_DIRECTORY: c_int = 0x10000;

const AT_FDCWD: c_int = -100;
const BUF_SIZE: usize = 4096 * 16; 

const SEEK_END  : c_int = 2;
const DT_REG    : u8    = 8;
const DT_DIR    : u8    = 4;
const DT_LNK    : u8    = 10;

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum Access {
    Read,
    Write,
    Append,
    ReadWrite,
    ReadWriteNewFile,
    ReadAppendNewFile
}

impl Access {
    const fn as_cstr(self) -> &'static CStr {
        match self {
            Self::Read => c"r",
            Self::Write => c"w",
            Self::Append => c"a",
            Self::ReadWrite => c"r+",
            Self::ReadWriteNewFile => c"w+",
            Self::ReadAppendNewFile => c"a+",
        }
    }
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
        let path = path.into();
        let c_str = path.as_c_str();
        let file = fopen(c_str.as_ptr(), access.as_cstr().as_ptr());

        if file.is_null() {
            Err(ErrorCode::last())
        } else {
            Ok(Self(file))
        }
    }

    pub fn create_new(path: impl Into<Path>, access: Access) -> error::Result<Self> {
        let new_access = match access {
            Access::Read | Access::Write | Access::ReadWrite => Access::ReadWriteNewFile,
            Access::Append => Access::ReadAppendNewFile,
            _ => access
        };
        Self::open(path, new_access)
    }

    pub fn create_always(path: impl Into<Path>, access: Access) -> error::Result<Self> {
        Self::create_new(path, access)
    }

    pub const fn as_handle(&self) -> FileHandle {
        self.0
    }

    pub fn write(&self, buf: impl Into<Vec<u8>>) -> error::Result<()> {
        let buf = buf.into();
        let written = fwrite(buf.as_ptr().cast(), 1, buf.len(), self.0);
        if written == buf.len() {
            Ok(())
        } else {
            Err(ErrorCode::last())
        }
    }

    pub fn read(&self) -> error::Result<Vec<u8>> {
        rewind(self.0);
        let mut buf = Vec::with_capacity(4096);
        let mut chunk = [0u8; 4096];

        loop {
            let n = fread(chunk.as_mut_ptr().cast(), 1, chunk.len(), self.0);
            if n == 0 {
                if ferror(self.0) != 0 {
                    return Err(ErrorCode::last());
                }
                break;
            }
            buf.extend_from_slice(&chunk[..n]);
        }

        Ok(buf)
    }
}

impl Drop for File {
    fn drop(&mut self) {
        fclose(self.0);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemType {
    File, Dir, Link, Unknown
}

impl From<u8> for ItemType {
    fn from(value: u8) -> Self {
        match value {
            DT_REG => Self::File,
            DT_DIR => Self::Dir,
            DT_LNK => Self::Link,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Item {
    typ: ItemType,
    name: String
}

impl Item {
    pub fn typ(&self) -> ItemType {
        self.typ.clone()
    }

    pub const fn name(&self) -> &String {
        &self.name
    }
    
    pub fn into_name(self) -> String {
        self.name
    }
}

pub struct ReadDirIter {
    dir: Dir
}

impl Iterator for ReadDirIter {
    type Item = Item;
    fn next(&mut self) -> Option<Self::Item> {
        crate::println!("Called ReadDirIter::next()");
        loop {
            crate::println!("Called libc readdir");
            let raw_item = readdir(&raw mut self.dir);
            if raw_item.is_null() {
                return None
            }

            // SAFETY: We check if the pointer is null, safe
            let item = unsafe { raw_item.read() };

            // SAFETY: Libc is guaranteed to return a valid string
            let c_str_name = unsafe { CStr::from_ptr(item.d_name.as_ptr()) };
            if c_str_name == c"." || c_str_name == c".." {
                continue;
            }

            let name = c_str_name.to_string_lossy().into_owned();
            let typ = ItemType::from(item.d_type);

            crate::println!("Return item");
            return Some(Item { typ, name });
        }
    }
}

pub fn read_dir(path: impl Into<Path>) -> error::Result<ReadDirIter> {
    let path = path.into();
    let c_path = path.as_c_str();

    let dir = opendir(c_path.as_ptr());
    if dir.is_null() {
        return Err(ErrorCode::last());
    }

    Ok(ReadDirIter { dir })
}

pub fn read_dir_all(path: impl Into<Path>) -> error::Result<Vec<Item>> {
    let path = path.into();
    let c_path = path.as_c_str();

    let fd = open(c_path.as_ptr(), O_RDONLY | O_DIRECTORY, 0);
    // crate::println!("Open: Fd: {fd}, Path: {c_path:?}, Error: {}:{}", ErrorCode::last(), ErrorCode::last().code());
    if fd < 0 {
        return Err(ErrorCode::last());
    }

    let mut items = Vec::with_capacity(20);
    let mut buf = vec![0u8; BUF_SIZE].into_boxed_slice();

    loop {
        let n = getdents64(fd, buf.as_mut_ptr().cast(), BUF_SIZE);
        if n < 0 {
            let err = ErrorCode::last();
            close(fd);
            return Err(err);
        }
        if n == 0 {
            break;
        }

        let mut pos = 0;
        while pos < n as usize {
            // SAFETY: Libc always returns a valid pointer
            let entry = unsafe {
                ptr::read_unaligned(buf.as_ptr().add(pos).cast::<LinuxDirent64>()) 
            };
            let reclen = entry.d_reclen as usize;
            if reclen == 0 {
                break;
            }

            // SAFETY: Libc is guaranteed to return a valid string
            let name_cstr = unsafe { CStr::from_ptr(entry.d_name.as_ptr()) };
            let name = name_cstr.to_string_lossy().into_owned();
            if name != "." && name != ".." {
                let typ = ItemType::from(entry.d_type);
                items.push(Item { typ, name });
            }
            pos += reclen;
        }
    }

    close(fd);
    Ok(items)
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
    let c_path = path.as_c_str();

    let ret = mkdir(c_path.as_ptr(), 0o755);
    if ret == -1 {
        let err = ErrorCode::last();
        if !err.is_already_exists() {
            return Err(err);
        }
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

pub fn read_link(path: impl Into<Path>, len: usize) -> Option<String> {
    let path = path.into();
    let c_path = path.as_c_str();
    let mut buf = vec![c_char::default(); len];

    let len = readlink(c_path.as_ptr(), buf.as_mut_ptr(), len - 1);
    if len == -1 {
        None
    } else {
        buf[len as usize] = 0;
        // SAFETY: Libs are guaranteed to store a valid cstr
        let c_str = unsafe { CStr::from_ptr(buf.as_ptr()) };
        Some(c_str.to_string_lossy().into_owned())
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::String;

    use crate::linux::fs::{self, Access, File};

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