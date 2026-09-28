use alloc::{borrow::Cow, string::String};

use crate::{cfg_if, superstr::ConcatStr};

cfg_if! {
    if #[cfg(target_os = "windows")] {
        mod windows;
    } else if #[cfg(target_os = "linux")] {
        mod linux;
    } else if #[cfg(target_os = "android")] {
        mod android;
    }
}

pub struct OsInfo {
    pub sysname: &'static str,
    pub name: &'static str,
    pub id: ConcatStr<2>,
    pub id_like: ConcatStr<2>,
    pub version: String,
    pub version_id: String,
    pub codename: Cow<'static, str>,
    pub variant: Cow<'static, str>,
    pub variant_id: Cow<'static, str>,
    pub nerd: char,
}

#[cfg(target_os = "android")]
pub fn get_id() -> ConcatStr<2> {
    ConcatStr::new(["android", ""])
}

#[cfg(target_os = "linux")]
pub fn get_id() -> ConcatStr<2> {
    use crate::parser::LineBased;

    let os_release = LineBased::parse_os_release().unwrap();
    let id = os_release.get_default("ID", "Unknown");
    ConcatStr::new([id, ""])
}

#[cfg(target_os = "windows")]
pub fn get_id() -> ConcatStr<2> {
    use crate::{superstr::ConcatStr, windows::env};

    let (_, _, build) = env::get_version();
    let version = OsInfo::version(build as i32);
    ConcatStr::<2>::new(["Windows ", version])
}
