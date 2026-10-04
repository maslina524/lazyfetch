use crate::{
    modules::Shell, str::SmolStr, windows::env
};

pub fn get() -> Shell {
    let Some(info) = env::get_shell() else {
        return Shell::default();
    };

    let exe_path = info.exe_path.clone();
    let process_name = info.exe_path
        .last()
        .map_or_default(SmolStr::from);

    let pid = info.pid;

    Shell { 
        process_name, 
        exe: SmolStr::empty(), 
        exe_name: SmolStr::empty(), 
        version: SmolStr::empty(), 
        pid, 
        pretty_name: SmolStr::empty(), 
        exe_path, 
        tty: SmolStr::from("-1") 
    }
}