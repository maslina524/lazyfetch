use alloc::{
    borrow::ToOwned,
    borrow::Cow
};

use crate::{
    warning,
    windows::env,
    windows::path::Path,
    detect::initsystem::InitSystemInfo
};

const NAME: &str = "smss";

impl InitSystemInfo {
    pub fn new() -> Self {
        let path = "C:/Windows/System32/smss.exe".to_owned();
        let version = match env::get_file_product_version(&path) {
            Ok(v) => v,
            Err(e) => {
                warning!("Failed to get file version: {e} (initsystem)");
                "0.0.0.0".to_owned()
            }
        };

        Self { 
            exe: Path::from(path),
            name: Cow::Borrowed(NAME), 
            version
        }
    }
}