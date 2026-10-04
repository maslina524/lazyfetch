use crate::{
    modules::Shell,
    str::SmolStr,
    windows::env,
    warning
};

pub fn get() -> Shell {
    let Some(info) = env::get_shell() else {
        return Shell::default();
    };

    let exe_path = info.exe_path.clone();
    let process_name = info.exe_path
        .last()
        .map_or_default(SmolStr::from);

    let version = match env::get_file_product_version(&exe_path) {
        Ok(v) => v,
        Err(e) => {
            warning!("Failed to get file version: {e} (initsystem)");
            SmolStr::from_static("0.0.0.0")
        }
    };

    let pid = info.pid;
    let exe_name = process_name.clone();
    let exe = exe_path.clone();

    Shell { 
        process_name,
        exe,
        exe_name,
        version,
        pid,
        pretty_name: SmolStr::empty(),
        exe_path,
        tty: SmolStr::from("-1")
    }
}