use core::ffi::CStr;

use alloc::{
    borrow::{Cow, ToOwned},
    string::String,
};

use crate::{
    detect::initsystem::InitSystemInfo,
    str::SmolStr,
    unix::{
        fs::{self, ItemType},
        libc::__system_property_get,
        path::Path,
    },
};

const PROP_VALUE_MAX: usize = 92;

impl InitSystemInfo {
    pub fn new() -> Self {
        let (name, exe) = Self::name_and_exe();
        let mut c_version = [0u8; PROP_VALUE_MAX + 1];
        __system_property_get(c"ro.system.build.id".as_ptr(), c_version.as_mut_ptr());

        // SAFETY: Returns valid ptr
        let version_cow = unsafe { CStr::from_ptr(c_version.as_ptr().cast()) }.to_string_lossy();

        let version = SmolStr::from(version_cow);

        Self {
            exe,
            name: Cow::Owned(name),
            version,
        }
    }

    fn name_and_exe() -> (String, Path) {
        Self::read_rc().unwrap_or_else(|| ("Unknown".to_owned(), Path::new()))
    }

    fn read_rc() -> Option<(String, Path)> {
        let entries = fs::read_dir_all("/system/etc/init").ok()?;

        for entry in entries {
            #[allow(clippy::case_sensitive_file_extension_comparisons)]
            if entry.typ() == ItemType::File && entry.name().ends_with(".rc") {
                return Some((
                    "Android init (AOSP)".to_owned(),
                    Path::from("/system/etc/init"),
                ));
            }
        }

        None
    }
}
