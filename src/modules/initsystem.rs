use alloc::{
    string::String,
    boxed::Box,
    borrow::Cow
};
use doc::Docs;

use crate::{
    impl_display_for_module,
    impl_module,
    detect::initsystem::{InitSystemInfo, pid},
    modules::Module,
    ui::lazy::LazyField,
    sync::OnceLock,
    imp::path::Path
};

static INITSYSTEM: OnceLock<Initsystem> = OnceLock::new();

#[derive(Debug, Docs)]
pub struct Initsystem {
    #[doc = "Name"]
    pub name: Cow<'static, str>,
    #[doc = "Exe path"]
    pub exe: Path,
    #[doc = "Version path"]
    pub version: String,
    #[doc = "Pid"]
    pub pid: LazyField<u32>
}

impl Module for Initsystem {
    fn new() -> Self {
        let info = InitSystemInfo::new();
        Self {
            name: info.name,
            exe: info.exe,
            version: info.version,
            pid: LazyField::new(pid)
        }
    }

    fn get() -> &'static Self {
        INITSYSTEM.get_or_init(|| {
            Self::new()
        })
    }

    fn key(&self) -> &'static str {
        "InitSystem"
    }

    fn title(&self) -> &'static str {
        "{name} {version}"
    }

    fn string_name(&self) -> &'static str {
        "initsystem"
    }

    impl_module!(
        name, exe, version, pid
    );
}

impl_display_for_module!(Initsystem);