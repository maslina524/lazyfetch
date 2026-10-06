use core::ffi::CStr;

use alloc::{
    borrow::{ToOwned, Cow},
    string::String
};

use crate::{
    str::SmolStr,
    unix::{
        fs,
        libc::__system_property_get
    },
    detect::os::{OsInfo, get_id}
};

const PROP_VALUE_MAX: usize = 92;
const SYSNAME       : &str  = "Linux";
const ID            : &str  = "android";

impl OsInfo {
    pub fn new() -> Self {
        let mut c_version = [0u8; PROP_VALUE_MAX + 1];
        __system_property_get(c"ro.build.version.release".as_ptr(), c_version.as_mut_ptr());
        let version_raw = CStr::from_bytes_until_nul(&c_version)
            .unwrap()
            .to_string_lossy()
            .into_owned();

        let version = SmolStr::from(version_raw);

        let name = Cow::Borrowed("Android");

        Self { 
            sysname: SYSNAME,
            name,
            id: get_id(),
            id_like: get_id(),
            version: version.clone(),
            version_id: version,
            codename: Cow::Borrowed(""),
            variant: Cow::Borrowed(""),
            variant_id: Cow::Borrowed(""),
            nerd: '\u{f17b}'
        }
    }
    
    fn get_version(id: &str) -> Option<String> {
        match id {
            "debian" => fs::read_to_string("/etc/debian_version").ok(),
            "alpine" => fs::read_to_string("/etc/alpine-version").ok(),
            "rhel" | "centos" | "fedora" | "rocky" | "almalinux" => {
                let content = fs::read_to_string("/etc/redhat-release").ok()?;
                Some(Self::extract_version(&content))
            },
            "gentoo" => {
                let content = fs::read_to_string("/etc/gentoo-release").ok()?;
                Some(Self::extract_version(&content))
            },
            "arch" => Some("Rolling".to_owned()),
            _ => None // os-release -> VERSION_ID
        }
    }

    fn extract_version(s: &str) -> String {
        let mut ret = String::with_capacity(8);
        let mut in_ret = false;

        for ch in s.chars() {
            if ch.is_numeric() && !in_ret {
                in_ret = true;
                ret.push(ch);
                continue;
            }

            if in_ret && (ch.is_numeric() || ch == '.') {
                ret.push(ch);
                continue;
            }
            break;
        }
        
        ret
    }
}