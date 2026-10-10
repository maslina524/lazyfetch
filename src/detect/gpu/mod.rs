use alloc::{borrow::ToOwned, string::String};

use crate::{
    cfg_if, format,
    formats::{Frequency, MemorySize, Temperature},
    nvidia::NvidiaLib,
};

cfg_if! {
    if #[cfg(target_os = "windows")] {
        mod windows;
    } else if #[cfg(target_os = "linux")] {
        mod linux;
    } else if #[cfg(target_os = "android")] {
        mod android;
    }
}

#[derive(Default)]
pub enum GpuType {
    #[default]
    Unknown,
    Discrete,
    BuiltIn,
}

impl core::fmt::Debug for GpuType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Unknown => write!(f, "\"Unknown\""),
            Self::Discrete => write!(f, "\"Discrete\""),
            Self::BuiltIn => write!(f, "\"Built-in\""),
        }
    }
}

impl GpuType {
    pub fn get_by_vendor_and_bus(vendor_id: u32, device_id: u32, pci_address: &str) -> Self {
        match vendor_id {
            0x10DE => Self::Discrete,
            0x8086 => Self::BuiltIn,
            0x1002 => {
                if pci_address.starts_with("0000:00:") {
                    let dev_str = format!("{:04x}", device_id);
                    if dev_str.starts_with("67")
                        || dev_str.starts_with("68")
                        || dev_str.starts_with("69")
                        || dev_str.starts_with("73")
                        || dev_str.starts_with("74")
                    {
                        return Self::Discrete;
                    }
                    return Self::BuiltIn;
                }
                Self::Discrete
            }
            _ => Self::Unknown,
        }
    }

    pub fn get_old(vendor: u32, memory: MemorySize) -> Self {
        if memory > MemorySize::Mb(256.0) && [0x10DE, 0x1002, 0x1022].contains(&vendor) {
            Self::Discrete
        } else {
            Self::BuiltIn
        }
    }
}

impl core::fmt::Display for GpuType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Unknown => write!(f, "Unknown"),
            Self::Discrete => write!(f, "Discrete"),
            Self::BuiltIn => write!(f, "Built-in"),
        }
    }
}

#[derive(Default)]
pub struct GpuInfo {
    pub vendor_id: u32,
    pub vendor: &'static str,
    pub device_id: u32,
    pub driver: String,
    pub typ: GpuType,
    pub memory_total: MemorySize,
}

impl GpuInfo {
    const fn innotek_name(device_id: u32) -> &'static str {
        match device_id {
            0xBEEF => "VirtualBox Graphics Adapter",
            0xCAFE => "VirtualBox Guest Service",
            _ => "Unknown",
        }
    }

    const fn amd_name(device_id: u32) -> &'static str {
        match device_id {
            0x1114 => "Krackan",
            0x130F => "Kaveri",
            0x13C0 => "Granite Ridge",
            0x1435 => "Sephiroth",
            0x150E => "Strix",
            0x15BF => "Phoenix",
            0x15D8 => "Picasso",
            0x15DD => "Raven Ridge",
            0x15E7 => "Barcelo",
            0x1636 => "Renoir",
            0x1638 => "Cezanne",
            0x163F => "VanGogh",
            0x164C => "Lucienne",
            0x164E => "Raphael",
            0x1681 => "Rembrandt",
            0x1900 | 0x1901 => "HawkPoint",
            _ => "Unknown",
        }
    }

    const fn intel_name(device_id: u32) -> &'static str {
        match device_id {
            0x0166 => "HD Graphics 4000",
            0x0412 => "HD Graphics 4600",
            0x0A16 => "HD Graphics 4400",
            0x5916 => "UHD Graphics 620",
            0x5912 | 0x3E9B | 0x3E92 => "UHD Graphics 630",
            0x8A52 => "Iris Plus Graphics",
            0x9BC4 | 0x9A49 | 0x46A6 => "Iris Xe Graphics",
            0x56A0 | 0x56A1 => "Arc A380",
            0x56B0 => "Arc A580",
            0x56C0 => "Arc A750",
            0x56D0 => "Arc A770",
            _ => "Unknown",
        }
    }

    const fn vmware_name(device_id: u32) -> &'static str {
        match device_id {
            0x0405 => "SVGA II",
            0x0710 => "SVGA",
            _ => "Unknown",
        }
    }

    const fn vendor_name(vendor_id: u32) -> &'static str {
        match vendor_id {
            0x10DE => "NVIDIA",
            0x1002 | 0x1022 => "AMD",
            0x8086 => "Intel",
            0x1414 => "Microsoft (Software/WARP)",
            0x5143 => "Qualcomm",
            0x15ad => "VMware",
            _ => "Unknown",
        }
    }
}

pub fn temperature(vendor_id: u32) -> Temperature {
    let val = match vendor_id {
        0x10DE => NvidiaLib::get().gpu_temperature() as f32,
        _ => 0.0,
    };
    Temperature::Celsius(val)
}

#[cfg(not(target_os = "android"))]
pub fn name(vendor_id: u32, device_id: u32) -> String {
    match vendor_id {
        0x10DE => NvidiaLib::get().device_name(),
        0x15ad => format!("VMware {}", GpuInfo::vmware_name(device_id)),
        0x1002 => format!("AMD {}", GpuInfo::amd_name(device_id)),
        0x8086 => format!("Intel {}", GpuInfo::intel_name(device_id)),
        0x80EE => format!(
            "InnoTek Systemberatung GmbH {}",
            GpuInfo::innotek_name(device_id)
        ),
        _ => "Unknown".to_owned(),
    }
}

#[cfg(target_os = "android")]
pub fn name(_: u32, _: u32) -> String {
    crate::imp::fs::read_to_string("/sys/class/kgsl/kgsl-3d0/gpu_model")
        .unwrap_or_else(|_| "Unknown".to_owned())
}

pub fn frequency(vendor_id: u32) -> Frequency {
    let val = match vendor_id {
        0x10DE => NvidiaLib::get().get_frequency_ghz() as f32,
        _ => 0.0,
    };
    Frequency::GHz(val)
}
