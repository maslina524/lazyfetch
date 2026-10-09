use core::{ffi::c_void, mem::size_of, ptr};

use alloc::{string::String, vec, vec::Vec};

use crate::{
    sync::OnceLock,
    warning,
    windows::encoding::{Utf16Len, utf16le_to_utf8},
    windows::error::ErrorCode,
    windows::link::{GetRawInputDeviceInfoW, GetRawInputDeviceList, RAWINPUTDEVICELIST},
};

type DeviceHandle = *mut c_void;

const RIM_TYPEMOUSE: u32 = 0;
const RIM_TYPEKEYBOARD: u32 = 1;
const RIM_TYPEHID: u32 = 2;

const RIDI_DEVICENAME: u32 = 0x2000_0007;

#[derive(Debug)]
pub struct Keyboard {
    handle: DeviceHandle,
    data: OnceLock<KeyboardData>,
}

impl Keyboard {
    pub const fn from_handle(handle: DeviceHandle) -> Self {
        Self {
            handle,
            data: OnceLock::new(),
        }
    }

    pub fn get_data(&'_ self) -> &'_ KeyboardData {
        self.data.get_or_init(|| KeyboardData::new(self.handle))
    }
}

#[derive(Debug)]
pub struct KeyboardData {
    pub name: String,
}

impl KeyboardData {
    pub fn new(handle: DeviceHandle) -> Self {
        let mut buf = [0u16; 256];
        let mut buf_size = 255;

        // SAFETY: Completely safe
        let ret = unsafe {
            GetRawInputDeviceInfoW(
                handle,
                RIDI_DEVICENAME,
                buf.as_mut_ptr().cast(),
                &raw mut buf_size,
            )
        };
        if ret == u32::MAX {
            warning!(
                "GetRawInputDeviceInfoW failed: {} (name)",
                ErrorCode::last()
            );
            return Self {
                name: String::from("Unknown"),
            };
        }

        let name = utf16le_to_utf8(&buf, Utf16Len::NullTerminated).unwrap_or_else(|e| {
            warning!("Non Utf8 device name: {e}");
            String::from("Unknown")
        });

        Self { name }
    }
}

#[derive(Debug)]
pub enum Device {
    Mouse,
    Keyboard(Keyboard),
    Hid,
}

impl Device {
    pub const fn from_type(typ: u32, handle: DeviceHandle) -> Option<Self> {
        match typ {
            RIM_TYPEMOUSE => Some(Self::Mouse),
            RIM_TYPEKEYBOARD => Some(Self::Keyboard(Keyboard::from_handle(handle))),
            RIM_TYPEHID => Some(Self::Hid),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct DeviceManager {
    devices: Vec<Device>,
}

impl DeviceManager {
    pub fn new() -> Self {
        let mut count = 0;
        // SAFETY: Completely safe
        let ret = unsafe {
            GetRawInputDeviceList(
                ptr::null_mut(),
                &raw mut count,
                size_of::<RAWINPUTDEVICELIST>() as u32,
            )
        };
        if ret == u32::MAX {
            warning!("GetRawInputDeviceList failed: {} (size)", ErrorCode::last());
            return Self {
                devices: Vec::new(),
            };
        }
        if count == 0 {
            warning!("No devices found");
            return Self {
                devices: Vec::new(),
            };
        }

        let mut raw_devices = vec![RAWINPUTDEVICELIST::default(); count as usize];
        // SAFETY: Completely safe
        let ret = unsafe {
            GetRawInputDeviceList(
                raw_devices.as_mut_ptr(),
                &raw mut count,
                size_of::<RAWINPUTDEVICELIST>() as u32,
            )
        };
        if ret == u32::MAX {
            warning!("GetRawInputDeviceList failed: {} (data)", ErrorCode::last());
            return Self {
                devices: Vec::new(),
            };
        }

        let mut devices = Vec::with_capacity(count as usize);
        for d in raw_devices {
            if let Some(device) = Device::from_type(d.dwType, d.hDevice) {
                devices.push(device);
            }
        }

        Self { devices }
    }

    pub const fn as_devices(&self) -> &Vec<Device> {
        &self.devices
    }
}

#[cfg(test)]
mod tests {
    use crate::windows::device::{Device, DeviceManager};

    #[test]
    fn get_keyboards_test() {
        let manager = DeviceManager::new();
        for d in manager.as_devices() {
            if let Device::Keyboard(k) = d {
                let data = k.get_data();
                println!("{data:#?}");
            }
        }
    }
}
