use core::{mem, ptr};

use alloc::{
    borrow::{Cow, ToOwned},
    string::String,
    vec,
    vec::Vec,
};

use crate::{
    detect::cpu,
    formats::Frequency,
    modules::cpu::Cpu,
    windows::error::ErrorCode,
    windows::link::{
        GetActiveProcessorCount, GetLogicalProcessorInformation, GetNumaHighestNodeNumber,
        SYSTEM_LOGICAL_PROCESSOR_INFORMATION,
    },
    windows::regedit::{Access, Hkey, Regedit},
};

type LogicalInfo = SYSTEM_LOGICAL_PROCESSOR_INFORMATION;

pub fn get() -> Cpu {
    let cpu_regedit_handle = Regedit::open(
        Hkey::LocalMachine,
        "HARDWARE\\DESCRIPTION\\System\\CentralProcessor\\0",
        Access::Read,
    )
    .unwrap();

    let logical_info = logical_info();
    let vendor = cpu::vendor();
    let (family, model) = cpu::get_family_and_model();
    let code_name = cpu::code_name(&vendor, family, model);

    Cpu {
        name: Cow::Owned(name(&cpu_regedit_handle)),
        vendor,
        numa_nodes: numa_nodes_count(),
        cores_physical: physical_cores_count(&logical_info),
        cores_logical: logical_cores_count(&logical_info),
        cores_online: online_cores_count(),
        packages: package_count(&logical_info),
        code_name,
        technology: cpu::technology(),
        freq_base: base_freq(&cpu_regedit_handle),
        temperature: cpu::temperature(),
        freq_max: cpu::max_freq(),
        core_types: cpu::logical_grouped(),
        march: cpu::micro_arch(),
    }
}

fn logical_info() -> Vec<LogicalInfo> {
    let mut size = 0;
    // SAFETY: Completely safe
    unsafe { GetLogicalProcessorInformation(ptr::null_mut(), &raw mut size) };
    let err = ErrorCode::last();
    assert!(
        err.code() == 122 || err.code() == 0,
        "`GetLogicalProcessorInformation` (size) failed"
    );
    let struct_size = mem::size_of::<SYSTEM_LOGICAL_PROCESSOR_INFORMATION>();
    let buf_size = size as usize / struct_size;
    let mut buf = vec![SYSTEM_LOGICAL_PROCESSOR_INFORMATION::default(); buf_size];
    // SAFETY: Completely safe
    unsafe { GetLogicalProcessorInformation(buf.as_mut_ptr(), &raw mut size) };
    assert!(
        ErrorCode::last().code() != 0,
        "`GetLogicalProcessorInformation` (info) failed"
    );
    buf
}
fn numa_nodes_count() -> usize {
    let mut highest = 0;
    // SAFETY: Completely safe
    let ret = unsafe { GetNumaHighestNodeNumber(&raw mut highest) };
    if ret == 0 {
        return 0;
    }
    highest as usize + 1
}
fn physical_cores_count(buf: &[LogicalInfo]) -> usize {
    let mut physical = 0;
    for info in buf {
        if info.Relationship == 0 {
            physical += 1;
        }
    }
    physical
}
fn logical_cores_count(buf: &[LogicalInfo]) -> usize {
    let mut logical = 0;
    for info in buf {
        if info.Relationship == 0 {
            let mut mask = info.ProcessorMask;
            while mask != 0 {
                mask &= mask - 1;
                logical += 1;
            }
        }
    }
    logical
}
const fn package_count(buf: &[LogicalInfo]) -> usize {
    let mut package = 0;
    let mut idx = 0;
    while idx < buf.len() {
        let info = &buf[idx];
        if info.Relationship == 3 {
            package += 1;
        }
        idx += 1;
    }
    package
}
fn online_cores_count() -> usize {
    // SAFETY: Completely safe
    (unsafe { GetActiveProcessorCount(0) }) as usize
}
fn base_freq(handle: &Regedit) -> Frequency {
    let val = handle.read("~MHz").map_or_else(
        |_| 0,
        |key| {
            let mhz = key.as_u32().unwrap_or(0);
            mhz as u64 / 1000
        },
    );
    Frequency::from_hz(val)
}
fn name(handle: &Regedit) -> String {
    let reg = handle.read("ProcessorNameString").unwrap();
    let string = reg.as_string().unwrap();
    string.to_owned()
}

#[cfg(test)]
mod tests {
    use crate::{
        formats::Frequency,
        modules::{Module, cpu::Cpu},
    };

    #[test]
    fn vendor_test() {
        let info = Cpu::new();
        std::println!("Vendor: {}", info.vendor);
    }

    #[test]
    fn cores_test() {
        let info = Cpu::new();

        assert!(info.cores_physical != 0);
        assert!(info.cores_logical != 0);
        assert!(info.cores_online != 0);
    }

    #[test]
    fn numa_nodes_test() {
        let info = Cpu::new();
        assert!(info.numa_nodes != 0);
    }

    #[test]
    fn technology_test() {
        let info = Cpu::new();
        assert_ne!(info.technology, "");
    }

    #[test]
    fn base_freq_test() {
        let info = Cpu::new();
        assert_ne!(info.freq_base, Frequency::default());
    }

    #[test]
    fn micro_arch_test() {
        let info = Cpu::new();
        println!("{}", info.march);
    }

    #[test]
    fn package_test() {
        let info = Cpu::new();
        assert!(info.packages != 0);
    }
}
