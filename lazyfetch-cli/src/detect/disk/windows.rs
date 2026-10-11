use core::ptr;

use alloc::{string::String, vec::Vec};

use crate::{
    format,
    formats::{MemorySize, Percent, Time},
    modules::Disk,
    warning,
    windows::{
        error::ErrorCode,
        link::{
            GetDiskFreeSpaceExA, GetDriveTypeA, GetFileAttributesA, GetLogicalDrives,
            GetVolumeInformationA,
        },
    },
};

const INVALID_FILE_ATTRIBUTES: u32 = 0xFF_FF_FF_FF;
const FILE_ATTRIBUTE_HIDDEN: u32 = 0x02;
const DRIVE_REMOVABLE: u32 = 1;

pub fn get_disks_windows() -> Vec<Disk> {
    let mut ret = Vec::new();
    // SAFETY: Completely safe
    let mask = unsafe { GetLogicalDrives() };
    for i in 0..26 {
        if mask & (1 << i) == (1 << i) {
            let disk = process_disk(char::from(b'A' + i));
            ret.push(disk);
        }
    }

    ret
}

fn process_disk(letter: char) -> Disk {
    let mountpoint = format!("{letter}:\\");
    let mp_cstr = [letter as u8, b':', b'\\', 0];

    let mut free = 0;
    let mut total = 0;
    let mut avaible = 0;
    // SAFETY: Completely safe
    let ret = unsafe {
        GetDiskFreeSpaceExA(
            mp_cstr.as_ptr(),
            &raw mut avaible,
            &raw mut total,
            &raw mut free,
        )
    };
    if ret == 0 {
        warning!("Failed to get {mountpoint} size");
    }
    let used = total - free;
    #[allow(
        clippy::cast_precision_loss,
        reason = "Mantissa is 52 bits, 2^52 = 4 petabytes"
    )]
    let percent = (used as f64 / total as f64).clamp(0.0, 1.0);

    // SAFETY: Completely safe
    let attr = unsafe { GetFileAttributesA(mp_cstr.as_ptr()) };
    let is_hidden = if attr == INVALID_FILE_ATTRIBUTES {
        warning!(
            "Failed to determine whether {mountpoint} is hidden or not: {}",
            ErrorCode::last()
        );
        false
    } else {
        attr & FILE_ATTRIBUTE_HIDDEN != 0
    };

    // SAFETY: Completely safe
    let is_external = unsafe { GetDriveTypeA(mp_cstr.as_ptr()) } == DRIVE_REMOVABLE;

    let mut name_buf = [0u8; 256 + 1];
    let mut fs_buf = [0u8; 64 + 1];
    // SAFETY: Completely safe
    let ret = unsafe {
        GetVolumeInformationA(
            mp_cstr.as_ptr(),
            name_buf.as_mut_ptr(),
            256,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            fs_buf.as_mut_ptr(),
            64,
        )
    };
    if ret == 0 {
        warning!(
            "Failed to get label & fs name of {mountpoint}: {}",
            ErrorCode::last()
        );
    }
    let name_len = name_buf
        .iter()
        .position(|&c| c == 0 || c == 10)
        .unwrap_or(name_buf.len());
    let name = String::from_utf8_lossy(&name_buf[..name_len]).into_owned();
    let fs_len = fs_buf
        .iter()
        .position(|&c| c == 0 || c == 10)
        .unwrap_or(fs_buf.len());
    let filesystem = String::from_utf8_lossy(&fs_buf[..fs_len]).into_owned();

    Disk {
        size_used: MemorySize::from_bytes(used),
        size_total: MemorySize::from_bytes(total),
        size_percentage: Percent::new((percent * 100.0) as u8),
        files_used: 0,
        files_total: 0,
        files_percentage: Percent::default(),
        is_external,
        is_hidden,
        filesystem,
        name,
        is_readonly: false,
        create_time: Time::new(0),
        size_percentage_bar: String::new(),
        files_percentage_bar: String::new(),
        days: 0,
        hours: 0,
        minutes: 0,
        seconds: 0,
        milliseconds: 0,
        mountpoint,
        mount_from: String::new(),
    }
}
