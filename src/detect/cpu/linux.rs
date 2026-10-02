use alloc::{
    borrow::Cow, 
    vec::Vec
};

use crate::{
    abort,
    modules::cpu::Cpu,
    detect::cpu,
    format,
    formats::Frequency,
    imp::{fs, path::Path},
    parser::{LineBased, parse_range_notation},
};

pub fn get() -> Cpu {
    let info = LineBased::parse_cpu_info()
        .unwrap_or_else(|e| abort!("Failed to open /proc/cpuinfo: {e}"));

    let name = info.get_default("model name", "Unknown");
    let base_freq_raw = info
        .get("cpu MHz")
        .map_or(0.0, |s| s.parse::<f64>().unwrap_or(0.0));

    let logical = logical_cores_count();
    let freq_base = Frequency::from_hz((base_freq_raw * 1000.0) as u64);
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
        freq_base,
        temperature: cpu::temperature(),
        freq_max: cpu::max_freq(),
        core_types: cpu::logical_grouped(),
        march: cpu::micro_arch(),
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