use core::ffi::CStr;

use alloc::{string::String, vec::Vec};

use crate::{
    formats::{MemorySize, Percent, Time},
    modules::disk::Disk,
    unix::{
        error::ErrorCode,
        libc::{Statvfs, Statx, getmntent, setmntent, statvfs, statx},
    },
    warning,
};

const ST_RDONLY: u64 = 0b1;
const STATX_BTIME: u32 = 0x0800;

const DAY_SEC: u64 = 60 * 60 * 24;
const HOUR_SEC: u64 = 60 * 60;
const MIN_SEC: u64 = 60;

static SKIP_TYPES: [&CStr; 25] = [
    c"proc",
    c"sysfs",
    c"tmpfs",
    c"devtmpfs",
    c"devpts",
    c"cgroup",
    c"cgroup2",
    c"pstore",
    c"securityfs",
    c"debugfs",
    c"tracefs",
    c"fusectl",
    c"mqueue",
    c"hugetlbfs",
    c"bpf",
    c"configfs",
    c"autofs",
    c"binfmt_misc",
    c"rpc_pipefs",
    c"nsfs",
    c"overlay",
    c"squashfs",
    c"ramfs",
    c"fuse.gvfsd-fuse",
    c"fuse.portal",
];

#[cfg(not(target_os = "android"))]
static SKIP_DIRS: [&CStr; 9] = [
    c"/proc",
    c"/sys",
    c"/dev",
    c"/run",
    c"/tmp",
    c"/var/lib/docker",
    c"/var/lib/containers",
    c"/snap",
    c"/boot/efi",
];

#[cfg(target_os = "android")]
static SKIP_DIRS: [&CStr; 33] = [
    c"/proc",
    c"/sys",
    c"/dev",
    c"/run",
    c"/tmp",
    c"/var/lib/docker",
    c"/var/lib/containers",
    c"/snap",
    c"/boot/efi",
    // Termux
    c"/system",
    c"/vendor",
    c"/product",
    c"/odm",
    c"/metadata",
    c"/apex",
    c"/mnt",
    c"/data",
    c"/cache",
    c"/efs",
    c"/persist",
    c"/firmware",
    c"/bt_firmware",
    c"/dsp",
    c"/config",
    c"/acct",
    c"/patch_hn",
    c"/cust",
    c"/version",
    c"/preload",
    c"/preas",
    c"/preavs",
    c"/bootstrap",
    c"/log",
];

fn is_skipped(typ: &CStr, mount: &CStr) -> bool {
    let m = mount.to_bytes();
    if SKIP_DIRS.iter().any(|v| {
        let v = v.to_bytes();
        !v.is_empty() && m.windows(v.len()).any(|w| w == v)
    }) {
        return true;
    }

    SKIP_TYPES.contains(&typ)
}

pub fn get_disks_linux() -> Vec<Disk> {
    let fp = setmntent(c"/proc/mounts".as_ptr(), c"r".as_ptr());
    if fp.is_null() {
        warning!("Failed to read /proc/mounts: {}", ErrorCode::last());
        return Vec::new();
    }

    let mut ret = Vec::with_capacity(8);
    let mut ent;
    loop {
        ent = getmntent(fp);
        if ent.is_null() {
            break;
        }

        // SAFETY: Checked for a null pointer
        let data = unsafe { &*ent };
        // SAFETY: Libc is guaranteed to return a valid c string
        let fs_cstr = unsafe { CStr::from_ptr(data.mnt_typeL) };
        // SAFETY: Libc is guaranteed to return a valid c string
        let mount_cstr = unsafe { CStr::from_ptr(data.mnt_dirL) };
        if is_skipped(fs_cstr, mount_cstr) {
            continue;
        }

        // SAFETY: Libc is guaranteed to return a valid c string
        let mount_from_cstr = unsafe { CStr::from_ptr(data.mnt_fsnameL) };
        // SAFETY: Libc is guaranteed to return a valid c string
        let mount_cstr = unsafe { CStr::from_ptr(data.mnt_dirL) };

        let disk = process_mount(mount_cstr, mount_from_cstr, fs_cstr);
        ret.push(disk);
    }

    ret
}

fn process_mount(mount: &CStr, mount_from: &CStr, mount_fs: &CStr) -> Disk {
    let filesystem_str = mount_fs.to_string_lossy().into_owned();
    let mount_from_str = mount_from.to_string_lossy().into_owned();
    let mountpoint_str = mount.to_string_lossy().into_owned();

    let mut statv = Statvfs::default();
    let ret = statvfs(mount.as_ptr(), &raw mut statv);
    if ret != 0 {
        warning!(
            "Failed to call statvfs({}): {}",
            mountpoint_str,
            ErrorCode::last()
        );
    }

    let fr = if statv.f_frsize == 0 {
        statv.f_bsize
    } else {
        statv.f_frsize
    };
    let total = statv.f_blocks.saturating_mul(fr);

    let free = statv.f_bfree.saturating_mul(fr);
    let used = total.saturating_sub(free);
    #[allow(clippy::cast_precision_loss)]
    let percent = (used as f64 / total as f64).clamp(0.0, 1.0);

    let is_readonly = statv.f_flag & ST_RDONLY != 0;

    let mut stx = Statx::default();
    let ret = statx(0, mount, 0, STATX_BTIME, &raw mut stx);
    let ct_int = if ret == 0
        && (stx.stx_mask & STATX_BTIME) == STATX_BTIME
        && stx.stx_btime.tv_sec > 685_065_600
    {
        stx.stx_btime.tv_sec
    } else {
        0
    };

    Disk {
        size_used: MemorySize::from_bytes(used),
        size_total: MemorySize::from_bytes(total),
        size_percentage: Percent::new((percent * 100.0) as u8),
        files_used: 0,
        files_total: 0,
        files_percentage: Percent::default(),
        is_external: false,
        is_hidden: false,
        filesystem: filesystem_str,
        name: String::new(),
        is_readonly,
        create_time: Time::new(ct_int),
        size_percentage_bar: String::new(),
        files_percentage_bar: String::new(),
        days: (ct_int / DAY_SEC) as u32,
        hours: (ct_int / HOUR_SEC) as u8,
        minutes: (ct_int / MIN_SEC) as u8,
        seconds: (ct_int % MIN_SEC) as u8,
        milliseconds: 0,
        mountpoint: mountpoint_str,
        mount_from: mount_from_str,
    }
}
