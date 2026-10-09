use doc::Docs;

use crate::{
    detect::terminal, imp::path::Path, impl_display_for_module, impl_module, modules::Module,
    str::SmolStr, sync::OnceLock,
};

static TERMINAL: OnceLock<Terminal> = OnceLock::new();

#[derive(Debug, Default, Docs)]
pub struct Terminal {
    pub process_name: SmolStr,
    pub exe: Path,
    pub exe_name: SmolStr,
    pub version: SmolStr,
    pub pid: u32,
    pub pretty_name: SmolStr,
    pub exe_path: Path,
    pub tty: i32,
}

impl Module for Terminal {
    fn new() -> Self {
        terminal::get()
    }

    fn get() -> &'static Self {
        TERMINAL.get_or_init(Self::new)
    }

    fn key(&self) -> &'static str {
        "Terminal"
    }

    fn title(&self) -> &'static str {
        "{pretty-name} {version}"
    }

    fn string_name(&self) -> &'static str {
        "terminal"
    }

    impl_module!(
        process_name,
        exe,
        exe_name,
        version,
        pid,
        pretty_name,
        exe_path,
        tty
    );
}

impl_display_for_module!(Terminal);
