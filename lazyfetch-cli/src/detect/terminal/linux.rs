use core::convert::Into;

use crate::{
    detect::shell::get_shell_pid,
    format,
    modules::Terminal,
    str::SmolStr,
    unix::{
        error,
        fs::{self, ReadError},
        libc::c_pid,
        path::Path,
    },
    warning,
};

static BLACKLIST: [&str; 6] = [
    "python",
    "python3",
    "lazyfetch",
    "fastfetch",
    "neofetch",
    "cargo",
];

#[derive(Default)]
struct Process {
    pid: c_pid,
    name: SmolStr,
    arg: Path,
    exe_path: Path,
    tty: i32,
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

        Ok(Self {
            pid,
            name: SmolStr::from(name),
            arg,
            exe_path,
            tty,
        })
    }
}

fn get_terminal_process(shell_pid: u32) -> Result<Process, ReadError> {
    let state = fs::read_to_string(format!("/proc/{shell_pid}/stat"))?;
    let item_iter = state.split(' ');

    let mut item_iter = item_iter.skip(3); // pid comm state
    let terminal_pid = item_iter
        .next()
        .and_then(|s| s.parse::<i32>().ok())
        .expect("Strange unix /proc/pid/state");

    // Terminal process
    let state = fs::read_to_string(format!("/proc/{terminal_pid}/stat"))?;
    let mut item_iter = state.split(' ').skip(1); // pid
    let raw = item_iter.next().expect("Strange unix /proc/pid/stat");
    let name = if raw.starts_with('(') && raw.ends_with(')') {
        // Valid unix format
        &raw[1..raw.len() - 1]
    } else {
        raw
    };

    Process::new(terminal_pid, name).map_err(Into::into)
}

pub fn get() -> Terminal {
    let info = match get_terminal_process(get_shell_pid()) {
        Ok(p) => p,
        Err(e) => {
            warning!("Failed to get terminal process: {e}");
            return Terminal::default();
        }
    };

    let exe_name = info.arg.last().map(SmolStr::from).unwrap_or_default();
    let pretty_name = info.name.clone();

    Terminal {
        process_name: info.name,
        exe: info.arg,
        exe_name,
        version: SmolStr::default(),
        pid: info.pid as u32,
        pretty_name,
        exe_path: info.exe_path,
        tty: info.tty,
    }
}
