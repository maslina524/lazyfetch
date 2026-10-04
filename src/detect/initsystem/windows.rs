use alloc::{
    borrow::ToOwned,
    borrow::Cow
};

use crate::{
    detect::initsystem::InitSystemInfo, 
    str::SmolStr, 
    warning, 
    windows::{env, path::Path}
};

const NAME: &str = "smss";

impl InitSystemInfo {
    pub fn new() -> Self {
        let path = "C:/Windows/System32/smss.exe".to_owned();
        let version = match env::get_file_product_version(&path) {
            Ok(v) => v,
            Err(e) => {
                warning!("Failed to get file version: {e} (initsystem)");
                SmolStr::from_static("0.0.0.0")
            }
        };

        Self { 
            exe: Path::from(path),
            name: Cow::Borrowed(NAME), 
            version
        }
    }
}