use core::ffi::CStr;

use crate::{modules::Board, str::SmolStr, unix::libc::__system_property_get, warning};

static PROP_NAMES: [&CStr; 21] = [
    c"ro.build.version.magic",
    c"ro.build.version.magic.name",
    c"ro.build.version.emui",
    c"hw_sc.build.platform.version",
    c"ro.mi.os.version.name",
    c"ro.mi.os.version.incremental",
    c"ro.miui.ui.version.name",
    c"ro.build.version.oneui",
    c"ro.build.version.opporom",
    c"ro.build.version.oplusrom",
    c"ro.oplus.version",
    c"ro.build.version.realme",
    c"ro.vivo.os.name",
    c"ro.vivo.os.version",
    c"ro.build.version.zenui",
    c"ro.nothing.os.version",
    c"ro.build.version.release",
    c"ro.build.version.sdk",
    c"ro.build.display.id",
    c"ro.build.version.incremental",
    c"ro.build.version.security_patch",
];

const PROP_VALUE_MAX: usize = 92;

fn get_prop(name: &CStr) -> SmolStr {
    let mut buf = [0u8; PROP_VALUE_MAX + 1];
    __system_property_get(name.as_ptr(), buf.as_mut_ptr());
    let c_str = unsafe { CStr::from_ptr(buf.as_ptr().cast()) };
    SmolStr::try_from(c_str).unwrap_or_else(|e| {
        warning!("Failed to parse CStr: {e}");
        SmolStr::empty()
    })
}

fn get_version() -> SmolStr {
    for prop in PROP_NAMES {
        let mut buf = [0u8; PROP_VALUE_MAX + 1];
        let ret = __system_property_get(prop.as_ptr(), buf.as_mut_ptr());
        if ret == 0 {
            continue;
        }

        let c_str = unsafe { CStr::from_ptr(buf.as_ptr().cast()) };
        match SmolStr::try_from(c_str) {
            Ok(s) => return s,
            Err(e) => {
                warning!("Failed to parse CStr: {e}")
            }
        }
    }

    SmolStr::empty()
}

pub fn get() -> Board {
    let name = get_prop(c"ro.config.marketing_name");
    let vendor = get_prop(c"ro.product.brand");
    let version = get_version();

    Board {
        name,
        vendor,
        version,
        serial: SmolStr::empty(),
    }
}
