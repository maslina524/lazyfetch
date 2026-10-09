use core::sync::atomic::AtomicU32;

use crate::cfg_if;

pub static SHELL_PID: AtomicU32 = AtomicU32::new(0);

cfg_if! {
    if #[cfg(target_os = "windows")] {
        mod windows;
        pub use windows::{get, get_shell_pid};
    } else if #[cfg(target_family = "unix")] {
        mod unix;
        pub use unix::get;
    }
}

#[cfg(target_os = "linux")]
pub use unix::get_shell_pid;