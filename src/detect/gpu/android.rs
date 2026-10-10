use core::ffi::CStr;

use crate::{
    detect::gpu::{GpuInfo, GpuType},
    formats::MemorySize,
    unix::libc::__system_property_get,
};

const GPU_CLASSES: [&str; 6] = [
    "0x030000", "0x030100", "0x030200", "0x038000", "0x038100", "0x038200",
];
const PROP_VALUE_MAX: usize = 92;

impl GpuInfo {
    pub fn new() -> Self {
        let mut c_egl = [0u8; PROP_VALUE_MAX + 1];
        __system_property_get(c"ro.hardware.egl".as_ptr(), c_egl.as_mut_ptr());
        // SAFETY: Returns valid ptr
        let egl = unsafe { CStr::from_ptr(c_egl.as_ptr().cast()) }
            .to_string_lossy()
            .into_owned();

        let mut c_platform = [0u8; PROP_VALUE_MAX + 1];
        __system_property_get(c"ro.board.platform".as_ptr(), c_platform.as_mut_ptr());

        let (vendor, vendor_id) = match egl.as_str() {
            "adreno" => ("Qualcomm", 0x5143),
            "mali" => ("ARM", 0x13B5),
            "powervr" => ("Imagination", 0x1010),
            _ => ("Unknown", 0),
        };

        Self {
            vendor_id,
            vendor,
            device_id: 0,
            driver: egl,
            typ: GpuType::BuiltIn,
            memory_total: MemorySize::default(),
        }
    }
}
