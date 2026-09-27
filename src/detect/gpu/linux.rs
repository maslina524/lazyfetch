use alloc::{
    borrow::ToOwned,
    string::String
};

use crate::{
    detect::gpu::{GpuInfo, GpuType}, 
    linux::fs::{self, ItemType},
    linux::parser::LinuxInfo,
    formats::MemorySize,
    warning,
    format
};

const GPU_CLASSES: [&str; 6] = ["0x030000", "0x030100", "0x030200", "0x038000", "0x038100", "0x038200"];

impl GpuInfo {
    pub fn new() -> Self {
        let Some(pci_address) = Self::pci_address() else {
            return Self::default()
        };

        let vendor_id = Self::vendor_id(&pci_address);
        let device_id = Self::device_id(&pci_address);
        let driver = Self::driver(&pci_address)
            .map_or_else(String::new, ToOwned::to_owned);
        let memory_total = Self::memory_total();

        Self {
            vendor_id,
            vendor: Self::vendor_name(vendor_id),
            name: Self::name(vendor_id, device_id),
            device_id,
            driver,
            typ: GpuType::get_by_vendor_and_bus(vendor_id, device_id, &pci_address),
            memory_total,
            frequency: Self::frequency(vendor_id)
        }
    }

    #[todo::todo("Hard to implement on a VM, will be implemented later")]    
    fn memory_total() -> MemorySize {
        MemorySize::default()
    }

    fn device_id(pci_address: &str) -> u32 {
        let path = format!("/sys/bus/pci/devices/{pci_address}/device");
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                warning!("Failed to read {path}: {e}");
                return 0;
            }
        };
        let trimmed = content.trim_start_matches("0x").trim();

        u32::from_str_radix(trimmed, 16)
            .unwrap_or(0)
    }

    fn driver(pci_address: &str) -> Option<&'static str> {
        let path = format!("/sys/bus/pci/devices/{pci_address}/uevent");
        let parsed = match LinuxInfo::parse_file(&path, '=') {
            Ok(c) => c,
            Err(e) => {
                warning!("Failed to parse {path}: {e}");
                return None;
            }
        };

        parsed.get("DRIVER")
    }

    fn pci_address() -> Option<String> {
        let entries = match fs::read_dir_all("/sys/bus/pci/devices") {
            Ok(v) => v,
            Err(e) => {
                warning!("Failed to read devices: {e}");
                return None;
            }
        };

        for entry in entries {
            if entry.typ() != ItemType::Dir && entry.typ() != ItemType::Link {
                continue;
            }
            let name = entry.name();
            let content_path = format!("/sys/bus/pci/devices/{name}/class");
            if let Ok(content) = fs::read_to_string(content_path)
                && GPU_CLASSES.contains(&content.trim())
            {
                return Some(entry.into_name());
            }
        }

        warning!("Failed to find graphics card in devices");
        None
    }

    fn vendor_id(pci_address: &str) -> u32 {
        let content = fs::read_to_string(format!("/sys/bus/pci/devices/{pci_address}/vendor"))
            .unwrap_or_else(|e| { 
                warning!("Failed to read /sys/class/drm/card0/device/vendor: {e}");
                "0x0".to_owned()
            });
        let trimmed = content.trim_start_matches("0x").trim();

        u32::from_str_radix(trimmed, 16)
            .unwrap_or(0)
    }
}