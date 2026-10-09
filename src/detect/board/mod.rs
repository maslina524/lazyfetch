use crate::cfg_if;

cfg_if! {
    if #[cfg(target_os = "linux")] {
        mod linux;
        pub use linux::get;
    } else if #[cfg(target_os = "android")] {
        mod android;
        pub use android::get;
    }
}
