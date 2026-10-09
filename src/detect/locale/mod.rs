use alloc::string::String;

use crate::cfg_if;

cfg_if! {
    if #[cfg(target_os = "windows")] {
        mod windows;
        pub use windows::get;
    } else if #[cfg(target_family = "unix")] {
        mod unix;
        pub use unix::get;
    }
}

pub struct LocaleInfo {
    pub locale: String,
}
