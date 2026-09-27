use alloc::borrow::{ToOwned, Cow};

use crate::{
    warning,
    linux::fs,
    linux::path::Path,
    detect::initsystem::InitSystemInfo
};

impl InitSystemInfo {
    #[todo::todo("Too time-consuming to implement for every distribution and system")]
    pub fn new() -> Self {
        let name = fs::read_to_string("/proc/1/comm")
            .unwrap_or_else(|_| "Unknown".to_owned())
            .trim()
            .to_owned();

        let exe = fs::read_to_string("/proc/1/cmdline").map_or_else(|_| {
            warning!("Failed to read /proc/1/cmdline");
            Path::new()
        }, |cmdline| {
            let first_arg = cmdline.split('\0').next().unwrap_or("");
            if first_arg.is_empty() {
                Path::new()
            } else {
                fs::read_link(first_arg, 64).map_or(Path::new(), Path::from)
            }
        });

        let version = "0.0.0.0".to_owned();

        Self { 
            exe,
            name: Cow::Owned(name), 
            version
        }
    }
}