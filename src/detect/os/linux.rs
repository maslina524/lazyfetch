use alloc::{
    borrow::{ToOwned, Cow},
    string::String
};

use crate::{
    parser::LineBased,
    imp::fs,
    detect::os::OsInfo,
    str::ConcatStr
};

const SYSNAME: &str = "Linux";

impl OsInfo {
    pub fn new() -> Self {
        let os_release = LineBased::parse_os_release().unwrap();

        let name = os_release.get_default("NAME", "Unknown");
        let codename = os_release.get_default("VERSION_CODENAME", "");
        let variant = os_release.get_default("VARIANT", "");
        let variant_id = os_release.get_default("VARIANT_ID", "");
        let id = os_release.get_default("ID", "Unknown");
        let version = Self::get_version(id).unwrap_or_else(|| {
            os_release.get_default("VERSION_ID", "Unknown").to_owned()
        });
        let nerd = Self::nerd(id);

        Self { 
            sysname: SYSNAME,
            name,
            id: ConcatStr::new([id, ""]),
            id_like: ConcatStr::new([id, ""]),
            version: version.clone(),
            version_id: version,
            codename: Cow::Borrowed(codename),
            variant: Cow::Borrowed(variant),
            variant_id: Cow::Borrowed(variant_id),
            nerd
        }
    }

    fn nerd(id: &str) -> char {
        match id {
            "debian" => '\u{f08da}',
            "kali" => '\u{f327}',
            "ubuntu" => '\u{ef72}',
            "linuxmint" => '\u{f08ed}',
            "fedora" => '\u{e7d9}',
            "opensuse" | "sle" => '\u{ef6d}',
            "arch" => '\u{f08c7}',
            "manjaro" => '\u{f312}',
            "popos" => '\u{f32a}',
            "mxlinux" => '\u{f33f}',
            "almalinux" => '\u{e8f3}',
            "rockylinux" => '\u{e891}',
            "rhel" => '\u{ef5d}',
            "garuda" => '\u{f337}',
            "gentoo" => '\u{e7e6}',
            "slackware" => '\u{f318}',
            "nixos" => '\u{e843}',
            "raspios" => '\u{e722}',
            "zorin" => '\u{f32f}',
            "cachyos" => '\u{f385}',
            "void" => '\u{f32e}',
            _ => '\u{ebc6}'
        }
    }
    
    fn get_version(id: &str) -> Option<String> {
        match id {
            "debian" => fs::read_to_string("/etc/debian_version").map(|s| s.trim().to_owned()).ok(),
            "alpine" => fs::read_to_string("/etc/alpine-version").map(|s| s.trim().to_owned()).ok(),
            "rhel" | "centos" | "fedora" | "rocky" | "almalinux" => {
                let content = fs::read_to_string("/etc/redhat-release").map(|s| s.trim().to_owned()).ok()?;
                Some(Self::extract_version(&content))
            },
            "gentoo" => {
                let content = fs::read_to_string("/etc/gentoo-release").map(|s| s.trim().to_owned()).ok()?;
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