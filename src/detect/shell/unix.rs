use crate::{
    format,
    modules::Shell,
    str::SmolStr,
    unix::{
        error::{self, ErrorCode},
        fs::{self, ReadError},
        libc::{c_pid, getppid},
        path::Path
    },
    warning
};

static BLACKLIST: [&str; 5] = [
    "python", "python3", "lazyfetch",
    "fastfetch", "neofetch", "cargo"
];

#[derive(Default)]
struct Process {
    pid: c_pid,
    name: SmolStr,
    arg: Path,
    exe_path: Path,
    tty: i32
}

impl Process {
    pub fn new(pid: c_pid, name: &str) -> error::Result<Self> {
        let cmdline = fs::read(format!("/proc/{pid}/cmdline"))?;

        let arg = if cmdline.len() > 1 {
            let mut null_idx = 0;
            while null_idx < cmdline.len() && cmdline[null_idx] != 0 {
                null_idx += 1;
            }

            let slice = &cmdline[..null_idx];
            Path::from(str::from_utf8(slice).unwrap_or_default())
        } else {
            Path::default()
        };

        let exe = fs::read_link(format!("/proc/{pid}/exe"), 1024).unwrap_or_default();
        let exe_path = Path::from(exe);
        
        let tty_path = fs::read_link(format!("/proc/{pid}/fd/0"), 512).unwrap_or_default();
        let tty = Path::from(tty_path)
            .last()
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(-1);

        Ok(
            Self { pid, name: SmolStr::from(name), arg, exe_path, tty }
        )
    }
}

fn get_terminal_process() -> Result<Process, ReadError> {
    let mut pid = getppid();
    loop {
        let state = fs::read_to_string(format!("/proc/{pid}/stat"))?;
        let item_iter = state.split(' ');

        let mut item_iter = item_iter.skip(1); // pid
        let raw = item_iter.next().expect("Strange unix /proc/pid/state");

        let name = if raw.starts_with('(') && raw.ends_with(')') { // Valid unix format
            &raw[1..raw.len() - 1]
        } else {
            raw
        };
        
        if (0..=1).contains(&pid) || name.eq_ignore_ascii_case("MainThread") {
            return Process::new(pid, name).map_err(ErrorCode::into);
        }

        if !BLACKLIST.contains(&name) {
            return Process::new(pid, name).map_err(ErrorCode::into);
        }

        let mut item_iter = item_iter.skip(1); // state
        pid = item_iter
            .next()
            .and_then(|s| s.parse::<i32>().ok())
            .expect("Strange unix /proc/pid/state");
    }
}

pub fn get() -> Shell {
    let terminal = get_terminal_process()
        .unwrap_or_else(|e| {
            warning!("Failed to read /proc/pid/stat: {e}");
            Process::default()
        });
    
    let exe_name = terminal.arg
        .last()
        .map(SmolStr::from)
        .unwrap_or_default();

    let pretty_name = terminal.exe_path
        .last()
        .map(SmolStr::from)
        .unwrap_or_default();

    Shell { 
        process_name: terminal.name, 
        exe: terminal.arg, 
        exe_name, 
        version: SmolStr::default(), 
        pid: terminal.pid as u32, 
        pretty_name, 
        exe_path: terminal.exe_path, 
        tty: terminal.tty
    }
}