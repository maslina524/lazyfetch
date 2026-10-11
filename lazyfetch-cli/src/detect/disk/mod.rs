use alloc::vec::Vec;

use crate::{cfg_if, modules::Disk};

cfg_if! {
    if #[cfg(target_os = "windows")] {
        mod windows;
    } else if #[cfg(target_family = "unix")] {
        mod unix;
    }
}

pub fn get_disks() -> Vec<Disk> {
    cfg_if! {
        if #[cfg(target_os = "windows")] {
            windows::get_disks_windows()
        } else if #[cfg(target_family = "unix")] {
            unix::get_disks_linux()
        }
    }
}
