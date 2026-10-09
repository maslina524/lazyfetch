use core::ffi::CStr;

use crate::{
    modules::Terminal,
    str::SmolStr,
    unix::{libc::getenv, path::Path},
    warning,
};

static ENV_NAMES: [&CStr; 6] = [
    c"TERMUX_VERSION",
    c"TERMUX_APP_PACKAGE_MANAGER",
    c"TERMUX_APP__PACKAGE_NAME",
    c"TERMUX_MAIN_PACKAGE_FORMAT",
    c"TERMUX_APP__VERSION_NAME",
    c"TERMUX_APP__VERSION_CODE",
];

const TERMUX: &str = "Termux";

fn get_version() -> Option<SmolStr> {
    for env in ENV_NAMES {
        let ptr = getenv(env.as_ptr());
        if ptr.is_null() {
            continue;
        }

        // SAFETY: Valid ptr, safe
        let c_str = unsafe { CStr::from_ptr(ptr) };
        match SmolStr::try_from(c_str) {
            Ok(s) => return Some(s),
            Err(e) => {
                warning!("Failed to parse CStr: {e}");
            }
        }
    }

    None
}

pub fn get() -> Terminal {
    let version = get_version().unwrap_or_default();

    Terminal {
        process_name: SmolStr::from_static(TERMUX),
        exe: Path::new(),
        exe_name: SmolStr::empty(),
        version,
        pid: 0,
        pretty_name: SmolStr::from_static(TERMUX),
        exe_path: Path::new(),
        tty: -1,
    }
}
