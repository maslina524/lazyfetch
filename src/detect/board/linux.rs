use crate::{
    modules::Board,
    str::SmolStr,
    unix::{
        fs::{self, ReadError},
        path::Path,
    },
    warning,
};

const EACCES: i32 = 13;

fn read_file(path: Path) -> SmolStr {
    match fs::read_to_string(path) {
        Ok(s) => SmolStr::from(s.trim()),
        Err(ReadError::Code(e)) => {
            if e.code() != EACCES {
                warning!("Failed to read file: {e} ({})", e.code());
            }
            SmolStr::empty()
        }
        Err(ReadError::Utf8(e)) => {
            warning!("Failed to read file: {e}");
            SmolStr::empty()
        }
    }
}

pub fn get() -> Board {
    let root_path = Path::from("/sys/devices/virtual/dmi/id/");

    let name = read_file(root_path.join("board_name"));
    let vendor = read_file(root_path.join("board_vendor"));
    let version = read_file(root_path.join("board_version"));
    let serial = read_file(root_path.join("board_serial"));

    Board {
        name,
        vendor,
        version,
        serial,
    }
}
