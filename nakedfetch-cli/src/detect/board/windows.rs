use core::{ptr, slice};

use alloc::vec;

use crate::{modules::Board, warning, windows::link::GetSystemFirmwareTable};

const RSMB: u32 = 0x52_53_4D_42;
const OFF_MANUFACTURER: usize = 0x04;
const OFF_PRODUCT: usize = 0x05;
const OFF_VERSION: usize = 0x06;
const OFF_SERIAL: usize = 0x07;
const TYPE2_MIN_LEN: usize = 0x08;

#[allow(non_snake_case)]
#[repr(C)]
struct RawSMBIOSData {
    _Used20CallingMethod: u8,
    _SMBIOSMajorVersion: u8,
    _SMBIOSMinorVersion: u8,
    _DmiRevision: u8,
    _Length: u32,
    // Table Data
    // table: [u8]
}

fn str_at(table: &[u8], base: usize, struct_len: usize, idx: u8) -> &str {
    if idx == 0 {
        return "";
    }

    let mut p = base + struct_len;
    let mut i: u8 = 1;

    while p < table.len() && table[p] != 0 {
        let start = p;
        while p < table.len() && table[p] != 0 {
            p += 1;
        }
        if i == idx {
            return str::from_utf8(&table[start..p]).unwrap_or("");
        }
        p += 1; // Nul
        i += 1;
    }
    ""
}

pub fn get() -> Board {
    // SAFETY: Completely safe
    let size = unsafe { GetSystemFirmwareTable(RSMB, 0, ptr::null_mut(), 0) };
    if size == 0 {
        warning!("Failed to get buf size for board");
        return Board::default();
    }

    let mut buf = vec![0u8; size as usize];
    // SAFETY: Completely safe
    let ret = unsafe { GetSystemFirmwareTable(RSMB, 0, buf.as_mut_ptr(), size) };
    if ret == 0 {
        warning!("Failed to get board info");
        return Board::default();
    }

    let size_of_header = size_of::<RawSMBIOSData>();
    // SAFETY: Shift the pointer from the beginning of the structure to its "end",
    // where the table is located
    let ptr = unsafe { buf.as_mut_ptr().add(size_of_header) };
    // SAFETY: Create a slice from the beginning of the table to `size` - `size_of_header`
    let table = unsafe { slice::from_raw_parts(ptr, size as usize - size_of_header) };

    let mut pos = 0usize;
    while pos + 4 <= table.len() {
        let struct_type = table[pos];
        let struct_len = table[pos + 1] as usize;

        if struct_len < 4 || pos + struct_len > table.len() {
            break;
        }

        let mut next = pos + struct_len;
        while next + 1 < table.len() && !(table[next] == 0 && table[next + 1] == 0) {
            next += 1;
        }

        if struct_type == 2 && struct_len >= TYPE2_MIN_LEN {
            let base = pos;
            let vendor = str_at(table, base, struct_len, table[base + OFF_MANUFACTURER]);
            let product = str_at(table, base, struct_len, table[base + OFF_PRODUCT]);
            let version = str_at(table, base, struct_len, table[base + OFF_VERSION]);
            let serial = str_at(table, base, struct_len, table[base + OFF_SERIAL]);

            return Board {
                vendor: vendor.into(),
                name: product.into(),
                version: version.into(),
                serial: serial.into(),
            };
        }

        pos = next + 2;
    }

    warning!("Baseboard (SMBIOS Type 2) not found");
    Board::default()
}
