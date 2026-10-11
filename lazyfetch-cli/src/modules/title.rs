use alloc::string::String;
use doc::Docs;

use crate::{
    detect::title::TitleInfo, imp::path::Path, impl_display_for_module, impl_module,
    modules::Module, sync::OnceLock,
};

static TITLE: OnceLock<Title> = OnceLock::new();

#[derive(Debug, Docs)]
pub struct Title {
    #[doc = "User name"]
    pub user_name: String,
    #[doc = "Host name"]
    pub host_name: String,
    #[doc = "Home directory"]
    pub home_dir: Path,
    #[doc = "Executable path of current process"]
    pub exe_path: Path,
    #[doc = "User's default shell"]
    pub user_shell: Path,
    #[doc = "User name (colored)"]
    pub user_name_colored: String,
    #[doc = "@ symbol (colored)"]
    pub at_symbol_colored: &'static str,
    #[doc = "Host name (colored)"]
    pub host_name_colored: String,
    #[doc = "Full user name"]
    pub full_user_name: String,
    #[doc = "UID (*nix) / SID (Windows)"]
    pub user_id: String,
    #[doc = "PID of current process"]
    pub pid: u32,
    #[doc = "CWD with home dir replaced by `~`"]
    pub cwd: Path,
}

impl Module for Title {
    fn new() -> Self {
        let info = TitleInfo::new();
        Self {
            user_name: info.user_name,
            host_name: info.host_name,
            home_dir: info.home_dir,
            exe_path: info.exe_path,
            user_shell: info.user_shell,
            user_name_colored: info.user_name_colored,
            at_symbol_colored: "@",
            host_name_colored: info.host_name_colored,
            full_user_name: info.full_user_name,
            user_id: info.user_id,
            pid: info.pid,
            cwd: info.cwd,
        }
    }

    fn get() -> &'static Self {
        TITLE.get_or_init(Self::new)
    }

    fn key(&self) -> &'static str {
        ""
    }

    fn title(&self) -> &'static str {
        "{user-name-colored}{at-symbol-colored}{host-name-colored}"
    }

    fn string_name(&self) -> &'static str {
        "title"
    }

    impl_module!(
        user_name,
        host_name,
        home_dir,
        exe_path,
        user_shell,
        user_name_colored,
        at_symbol_colored,
        host_name_colored,
        full_user_name,
        user_id,
        pid,
        cwd
    );
}

impl_display_for_module!(Title);
