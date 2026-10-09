use crate::cfg_if;

cfg_if! {
    if #[cfg(target_os = "windows")] {
        mod windows;
        pub use windows::get;
    } else if #[cfg(target_os = "linux")] {
        mod linux;
        pub use linux::get;
    } else if #[cfg(target_os = "android")] {
        mod android;
        pub use android::get;
    }
}
