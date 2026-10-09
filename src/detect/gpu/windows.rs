use core::{ffi::c_void, mem, ptr, sync::atomic::AtomicPtr};

use alloc::{borrow::ToOwned, string::String};

use crate::{
    abort,
    detect::gpu::{GpuInfo, GpuType},
    formats::MemorySize,
    warning,
    windows::link::{
        CreateDXGIFactory, DIGCF_PRESENT, DXGI_ADAPTER_DESC, DXGI_ERROR_NOT_FOUND,
        GUID_DEVCLASS_DISPLAY, IDXGIAdapter_Vtbl, IDXGIFactory_Vtbl, IID_IDXGIFactory,
        SP_DEVINFO_DATA, SetupDiEnumDeviceInfo, SetupDiGetClassDevsW, SetupDiOpenDevRegKey,
    },
    windows::regedit::Regedit,
};

const INVALID_HANDLE: *mut c_void = (-1isize).cast_unsigned() as *mut c_void;

impl GpuInfo {
    pub fn new() -> Self {
        let desc =
            Self::dxgi_adapter_desc().unwrap_or_else(|e| abort!("CreateDXGIFactory error: {e}"));

        let driver = Self::driver_version().unwrap_or_else(|| {
            warning!("Failed to get driver version");
            String::from("Unknown")
        });

        let memory_total = MemorySize::from_bytes(desc.DedicatedVideoMemory as u64);

        Self {
            vendor_id: desc.VendorId,
            vendor: Self::vendor_name(desc.VendorId),
            device_id: desc.DeviceId,
            driver,
            typ: GpuType::get_old(desc.VendorId, memory_total),
            memory_total,
        }
    }

    fn dxgi_adapter_desc() -> Result<DXGI_ADAPTER_DESC, i32> {
        let mut factory_void = ptr::null_mut();

        // SAFETY: Completely safe
        let hr = unsafe { CreateDXGIFactory(&IID_IDXGIFactory, &raw mut factory_void) };

        if hr < 0 || factory_void.is_null() {
            return Err(hr);
        }

        // SAFETY: We check that the raw pointer is not null
        let factory_vtbl = unsafe { *factory_void.cast::<*mut IDXGIFactory_Vtbl>() };

        let mut i = 0;
        loop {
            let mut adapter_void = ptr::null_mut();

            // SAFETY: A virtual table is guaranteed to be located at the raw pointer
            let hr =
                unsafe { ((*factory_vtbl).EnumAdapters)(factory_void, i, &raw mut adapter_void) };
            if hr == DXGI_ERROR_NOT_FOUND {
                return Err(hr);
            }

            if !adapter_void.is_null() {
                let mut desc = DXGI_ADAPTER_DESC::default();

                // SAFETY: We check that the raw pointer is not null
                let adapter_vtbl = unsafe { *adapter_void.cast::<*mut IDXGIAdapter_Vtbl>() };
                // SAFETY: A virtual table is guaranteed to be located at the raw pointer
                let hr = unsafe { ((*adapter_vtbl).GetDesc)(adapter_void, &raw mut desc) };

                if hr >= 0 {
                    return Ok(desc);
                }
            }

            i += 1;
        }
    }

    fn driver_version() -> Option<String> {
        // SAFETY: Completely safe
        let handle = unsafe {
            SetupDiGetClassDevsW(
                &GUID_DEVCLASS_DISPLAY,
                ptr::null(),
                ptr::null_mut(),
                DIGCF_PRESENT,
            )
        };
        if handle == -1 {
            return None;
        }

        let mut info = SP_DEVINFO_DATA {
            cbSize: mem::size_of::<SP_DEVINFO_DATA>() as u32,
            ..SP_DEVINFO_DATA::default()
        };

        // SAFETY: Completely safe
        let ret = unsafe { SetupDiEnumDeviceInfo(handle, 0, &raw mut info) };
        if ret == 0 {
            return None;
        }

        // SAFETY: Completely safe
        let hkey = unsafe { SetupDiOpenDevRegKey(handle, &raw mut info, 1, 0, 2, 0x20019) };
        if hkey == INVALID_HANDLE {
            return None;
        }

        let reg = Regedit::from_handle(AtomicPtr::new(hkey));
        let value = reg.read("DriverVersion").ok()?;
        let string = value.as_string()?.to_owned();

        Some(string)
    }
}

#[cfg(test)]
mod tests {
    use crate::detect::gpu::GpuInfo;

    #[test]
    fn vendor_test() {
        let info = GpuInfo::new();
        let name = info.vendor;
        assert_ne!(name, "");
        println!("Vendor: {name}");
    }

    #[test]
    fn device_id_test() {
        let info = GpuInfo::new();
        let id = info.device_id;
        assert!(id != 0);
        println!("Id: {id}");
    }

    #[test]
    fn driver_test() {
        let info = GpuInfo::new();
        let driver = info.driver;
        assert!(driver != "Unknown");
        println!("Driver: {driver}");
    }
}
