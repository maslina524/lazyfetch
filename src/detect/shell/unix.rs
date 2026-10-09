use core::sync::atomic::Ordering::Relaxed;

use crate::{
    detect::shell::SHELL_PID,
    format,
    modules::Shell,
    str::SmolStr,
    sync::OnceLock,
    unix::{
        error, fs,
        libc::{c_pid, getppid},
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

static NAME: OnceLock<SmolStr> = OnceLock::new();

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

pub fn get_shell_pid() -> u32 {
    let loaded = SHELL_PID.load(Relaxed);
    crate::println!("SHELL: `{loaded}`");
    if loaded != 0 {
        return loaded;
    }

    let mut pid = getppid();
    loop {
        let state = match fs::read_to_string(format!("/proc/{pid}/stat")) {
            Ok(s) => s,
            Err(e) => {
                warning!("Failed to read /proc/pid/stat: {e}");
                return 0;
            }
        };
        let item_iter = state.split(' ');

        let mut item_iter = item_iter.skip(1); // pid
        let raw = item_iter.next().expect("Strange unix /proc/pid/state");

        let name = if raw.starts_with('(') && raw.ends_with(')') {
            // Valid unix format
            &raw[1..raw.len() - 1]
        } else {
            raw
        };
        let _ = NAME.set(SmolStr::from(name));

        if (0..=1).contains(&pid) || name.eq_ignore_ascii_case("MainThread") {
            SHELL_PID.store(pid as u32, Relaxed);
            return pid as u32;
        }

        if !BLACKLIST.contains(&name) {
            SHELL_PID.store(pid as u32, Relaxed);
            return pid as u32;
        }

        let mut item_iter = item_iter.skip(1); // state
        pid = item_iter
            .next()
            .and_then(|s| s.parse::<i32>().ok())
            .expect("Strange unix /proc/pid/state");
    }
}

pub fn get() -> Shell {
    let pid = get_shell_pid();
    let info = Process::new(pid as i32, NAME.get_or_abort("Unreachable")).unwrap_or_else(|e| {
        warning!("Failed to read /proc/pid/stat: {e}");
        Process::default()
    });

    let exe_name = info.arg.last().map(SmolStr::from).unwrap_or_default();
    let pretty_name = info.name.clone();

    Shell {
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
