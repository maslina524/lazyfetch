use core::ffi::CStr;

use alloc::{borrow::Cow, vec::Vec};

use crate::{
    abort,
    detect::cpu,
    format,
    formats::Frequency,
    modules::cpu::Cpu,
    parser::parse_range_notation,
    unix::{fs, libc::__system_property_get, path::Path},
    warning,
};

const PROP_VALUE_MAX: usize = 92;

pub fn get() -> Cpu {
    let mut c_tech = [0u8; PROP_VALUE_MAX + 1];
    __system_property_get(c"ro.soc.model".as_ptr(), c_tech.as_mut_ptr());
    let tech = unsafe { CStr::from_ptr(c_tech.as_ptr().cast()) };
    let name = tech_to_name(tech.to_bytes());
    let logical = logical_cores_count();
    let vendor = cpu::vendor();
    let (family, model) = cpu::get_family_and_model();
    let code_name = cpu::code_name(&vendor, family, model);

    Cpu {
        name: Cow::Borrowed(name),
        vendor,
        numa_nodes: numa_nodes_count(),
        cores_physical: physical_cores_count(),
        cores_logical: logical,
        cores_online: online_cores_count(logical),
        packages: package_count(),
        code_name,
        technology: cpu::technology(),
        freq_base: base_freq_android(),
        temperature: cpu::temperature(),
        freq_max: cpu::max_freq(),
        core_types: cpu::logical_grouped(),
        march: cpu::micro_arch(),
    }
}

fn max_freq_android() -> Frequency {
    match fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq") {
        Ok(c) => {
            let hz = c.trim().parse::<u64>().unwrap_or(0);
            Frequency::from_hz(hz)
        }
        Err(e) => {
            warning!("Failed to read .../cpufreq/cpuinfo_max_freq: {e}");
            Frequency::default()
        }
    }
}

fn base_freq_android() -> Frequency {
    match fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq") {
        Ok(c) => {
            let hz = c.trim().parse::<u64>().unwrap_or(0);
            Frequency::from_hz(hz)
        }
        Err(e) => {
            warning!("Failed to read .../cpufreq/scaling_cur_freq: {e}");
            Frequency::default()
        }
    }
}

fn tech_to_name(tech: &[u8]) -> &'static str {
    match tech {
        b"SM6225" => "Snapdragon 680 4G",
        b"SM6225-AD" => "Snapdragon 685",
        b"SM6115" => "Snapdragon 662/665/460",
        b"SM4250" => "Snapdragon 460",
        b"SM6375" => "Snapdragon 695 5G",
        b"SM8450" => "Snapdragon 8 Gen 1",
        b"SM8550" => "Snapdragon 8 Gen 2",
        _ => "Unknown",
    }
}

fn physical_cores_count() -> usize {
    let mut ret = Vec::with_capacity(24);
    let mut n = 0;
    loop {
        let path = Path::from(format!("/sys/devices/system/cpu/cpu{n}/topology/core_id"));
        if !path.exists() {
            break;
        }
        let content = fs::read_to_string(path).unwrap();
        let Ok(id) = content.trim().parse::<usize>() else {
            continue;
        };
        if !ret.contains(&id) {
            ret.push(id);
        }
        n += 1;
    }
    ret.len()
}

fn logical_cores_count() -> usize {
    let mut n = 0;
    loop {
        let path = Path::from(format!("/sys/devices/system/cpu/cpu{n}/"));
        if path.exists() {
            n += 1;
        } else {
            break;
        }
    }
    n
}

fn online_cores_count(logical_cores: usize) -> usize {
    let content = match fs::read_to_string("/sys/devices/system/cpu/online") {
        Ok(c) => c,
        Err(e) => abort!("Failed to read /sys/devices/system/cpu/online: {e}"),
    };

    let cores = parse_range_notation(&content, Some(logical_cores));
    cores.len()
}

fn package_count() -> usize {
    let mut ret = Vec::with_capacity(24);
    let mut n = 0;
    loop {
        let path = Path::from(format!(
            "/sys/devices/system/cpu/cpu{n}/topology/physical_package_id"
        ));
        if !path.exists() {
            break;
        }
        let content = fs::read_to_string(path).unwrap();
        let Ok(id) = content.trim().parse::<usize>() else {
            continue;
        };
        if !ret.contains(&id) {
            ret.push(id);
        }
        n += 1;
    }
    ret.len()
}

fn numa_nodes_count() -> usize {
    let mut n = 0;
    loop {
        let path = Path::from(format!("/sys/devices/system/node/node{n}"));
        if !path.exists() {
            break;
        }
        n += 1;
    }
    n
}
