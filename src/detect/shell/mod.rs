use crate::cfg_if;

cfg_if! {
    if #[cfg(target_os = "windows")] {
        mod windows;
        pub use windows::get;
    } else if #[cfg(any(target_os = "linux", target_os = "android"))] {
        mod linux;
        pub use linux::get;
    }
}

