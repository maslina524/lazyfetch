use core::ffi::{CStr, c_char, c_uint, c_void};

use alloc::{
    string::String,
    borrow::ToOwned
};

use crate::{
    abort, 
    sync::OnceLock, 
    warning,
    get_fn,
    cfg_if
};

cfg_if! {
    if #[cfg(target_os = "windows")] {
        use crate::windows::link::{FreeLibrary, HMODULE, LoadLibraryA};

        type ApiBaseFn = unsafe extern "system" fn() -> isize;
        type LibHandle = HMODULE;
    } else if #[cfg(any(target_os = "linux", target_os = "android"))] {
        use crate::linux::libc::{dlopen, dlclose};

        type ApiBaseFn = *mut c_void;
        type LibHandle = *mut c_void;
    }
}

#[allow(non_camel_case_types)]
type nvmlReturn = i32;
#[allow(non_camel_case_types)]
type nvmlTemperatureSensors = i32;
#[allow(non_camel_case_types)]
type nvmlDevice = *mut c_void;
#[allow(non_camel_case_types)]
type nvmlClockType = u32;

#[allow(non_camel_case_types)]
type nvmlInit = unsafe extern "C" fn() -> nvmlReturn;
#[allow(non_camel_case_types)]
type nvmlShutdown = unsafe extern "C" fn() -> nvmlReturn;
#[allow(non_camel_case_types)]
type nvmlDeviceGetHandleByIndex = unsafe extern "C" fn(index: c_uint, device: *mut nvmlDevice) -> nvmlReturn;
#[allow(non_camel_case_types)]
type nvmlDeviceGetTemperature = unsafe extern "C" fn(device: nvmlDevice, sensor: nvmlTemperatureSensors, temp: *mut c_uint) -> nvmlReturn;
#[allow(non_camel_case_types)]
type nvmlDeviceGetClockInfo = unsafe extern "C" fn(device: nvmlDevice, typ: nvmlClockType, clock: *mut u32) -> nvmlReturn;
#[allow(non_camel_case_types)]
type nvmlDeviceGetName = unsafe extern "C" fn(device: nvmlDevice, name: *mut c_char, length: c_uint) -> nvmlReturn;
#[allow(non_camel_case_types)]
type nvmlErrorString = unsafe extern "C" fn(result: nvmlReturn) -> *const c_char;

static NVIDIA: OnceLock<NvidiaLib> = OnceLock::new();

const NVML_CLOCK_SM: u32 = 1;
const NAME_BUFFER_SIZE: usize = 64;

pub struct NvidiaLib {
    handle: LibHandle,
    device: nvmlDevice,
    init: nvmlInit,
    shutdown: nvmlShutdown,
    device_get_handle_by_index: nvmlDeviceGetHandleByIndex,
    device_get_temperature: nvmlDeviceGetTemperature,
    get_clock_info: nvmlDeviceGetClockInfo,
    get_name: nvmlDeviceGetName
}

// SAFETY: Fields are never mutated after initialization
unsafe impl Sync for NvidiaLib {}

impl NvidiaLib {
    pub fn get() -> &'static Self {
        NVIDIA.get_or_init(|| {
            // Load library
            let lib = load();

            // SAFETY: `get_fn!` transmutes the resolved symbol to the declared fn type
            let init = unsafe { get_fn!(lib, c"nvmlInit_v2", nvmlInit) };
            // SAFETY: See above
            let shutdown = unsafe { get_fn!(lib, c"nvmlShutdown", nvmlShutdown) };
            // SAFETY: See above
            let device_get_handle_by_index = unsafe { get_fn!(lib, c"nvmlDeviceGetHandleByIndex_v2", nvmlDeviceGetHandleByIndex) };
            // SAFETY: See above
            let device_get_temperature = unsafe { get_fn!(lib, c"nvmlDeviceGetTemperature", nvmlDeviceGetTemperature) };
            // SAFETY: See above
            let get_clock_info = unsafe { get_fn!(lib, c"nvmlDeviceGetClockInfo", nvmlDeviceGetClockInfo) };
            // SAFETY: See above
            let get_name = unsafe { get_fn!(lib, c"nvmlDeviceGetName", nvmlDeviceGetName) };

            // SAFETY: FFI call, valid fn pointer resolved above
            let ret = unsafe { init() };
            if ret != 0 {
                abort!("Failed to initialize nvml");
            }

            let mut device = nvmlDevice::default();
            // SAFETY: FFI call, valid fn pointer resolved above
            let ret = unsafe { (device_get_handle_by_index)(0, &raw mut device) };
            if ret != 0 {
                // SAFETY: FFI call, valid fn pointer resolved above
                unsafe { (shutdown)() };
                abort!("Failed to get handle by index (nvml)");
            }

            Self {
                handle: lib,
                device,
                init, shutdown, 
                device_get_handle_by_index, device_get_temperature,
                get_clock_info,
                get_name
            }
        })
    }
    
    pub fn drop_nvidia() {
        if let Some(lib) = NVIDIA.get() {
            // SAFETY: FFI call, valid fn pointer resolved above
            unsafe { (lib.shutdown)() };
            // Intentionally do not unload the library
        }
    }

    pub fn gpu_temperature(&self) -> u16 {
        let mut temp = 0u32;

        // SAFETY: FFI call with a valid out pointer
        let ret = unsafe { (self.device_get_temperature)(self.device, 0, &raw mut temp) };
        if ret != 0 {
            return 0;
        }

        temp as u16
    }

    pub fn device_name(&self) -> String {
        let mut buf = [c_char::default(); NAME_BUFFER_SIZE + 1];
        // SAFETY: FFI call with a buffer of the declared size
        let ret = unsafe { (self.get_name)(self.device, buf.as_mut_ptr(), NAME_BUFFER_SIZE as u32) };
        if ret != 0 {
            warning!("Failed to get gpu name (nvml)");
            return "Unknown".to_owned();
        }

        // SAFETY: Иuffer is NUL-terminated by NVML on success
        let c_str = unsafe { CStr::from_ptr(buf.as_ptr()) };
        c_str.to_string_lossy().into_owned()
    }

    pub fn get_frequency_ghz(&self) -> f64 {
        let mut clock = 0;

        // SAFETY: FFI call with a valid out pointer
        let ret = unsafe {
            (self.get_clock_info)(self.device, NVML_CLOCK_SM, &raw mut clock)
        };
        
        if ret == 0 {
            clock as f64 / 100.0
        } else {
            warning!("Failed to get GPU frequency (nvml)");
            0.0
        }
    }
}

cfg_if! {
    if #[cfg(target_os = "windows")] {
        fn load() -> HMODULE {
            // SAFETY: LoadLibraryA with a NUL-terminated ASCII string
            let lib = unsafe {
                LoadLibraryA(c"nvml.dll".as_ptr().cast())
            };
            if lib.is_null() {
                abort!("Failed to load nvml.dll");
            }
            lib
        }

        fn unload(lib: HMODULE) {
            // SAFETY: Completely safe
            unsafe {
                FreeLibrary(lib)
            };
        }
    } else if #[cfg(any(target_os = "linux", target_os = "android"))] {
        fn load() -> LibHandle {
            let lib_names = [c"libnvidia-ml.so.1", c"libnvidia-ml.so"];

            for name in &lib_names {
                // SAFETY: dlopen with a NUL-terminated ASCII string
                let lib = dlopen(name.as_ptr().cast(), 1);
                if !lib.is_null() {
                    return lib;
                }
            }
            abort!("Failed to load nvml library")
        }

        fn unload(lib: LibHandle) {
            // SAFETY: Completely safe
            dlclose(lib);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::nvidia::NvidiaLib;

    #[test]
    fn get_temperature_test() {
        let temp = NvidiaLib::get().gpu_temperature();
        assert!(temp != 0);
        println!("Temperature: {temp}");
    }
}