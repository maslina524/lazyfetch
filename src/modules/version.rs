use doc::Docs;

use crate::{
    impl_module,
    impl_display_for_module,
    modules::Module,
    sync::OnceLock
};

static VERSION: OnceLock<Version> = OnceLock::new();

#[derive(Debug, Docs)]
pub struct Version {
    #[doc = "Project name"]
    pub project_name: &'static str,
    #[allow(clippy::struct_field_names)]
    #[doc = "Version"]
    pub version: &'static str,
    #[allow(clippy::struct_field_names)]
    #[doc = "Version tweak"]
    pub version_tweak: &'static str,
    #[doc = "Build type (debug or release)"]
    pub build_type: &'static str,
    #[doc = "System name"]
    pub sysname: &'static str,
    #[doc = "Architecture"]
    pub arch: &'static str,
    #[doc = "Always empty"]
    pub cmake_built_type: &'static str,
    #[doc = "Date time when compiling, like `Sep 03 2026, 22:09:40`"]
    pub compile_time: &'static str,
    #[doc = "Rustc version, like `rustc 1.97.1`"]
    pub compiler: &'static str,
    #[doc = "Glibc version, always empty on windows"]
    pub libc: &'static str,
    #[doc = "Cargo version, like `cargo 1.97.1`"]
    pub package_manager: &'static str,
    #[doc = "Link to the release of this version of lazyfetch"]
    pub release_link: &'static str
}

impl Module for Version {
    fn new() -> Self {
        let build_type = if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        };
        Self {
            project_name: env!("CARGO_BIN_NAME"), 
            version: env!("CARGO_PKG_VERSION"), 
            version_tweak: "", 
            build_type, 
            sysname: env!("TARGET_OS"),
            arch: env!("TARGET_ARCH"), 
            cmake_built_type: "", 
            compile_time: env!("COMPILE_TIME"), 
            compiler: env!("RUSTC_VERSION"), 
            libc: env!("LIBC_VERSION"),
            package_manager: env!("CARGO_VERSION"),
            release_link: concat!("https://github.com/maslina524/lazyfetch/releases/tag/v", env!("CARGO_PKG_VERSION"))
        }
    }

    fn get() -> &'static Self {
        VERSION.get_or_init(|| {
            Self::new()
        })
    }

    fn key(&self) -> &'static str {
        "Version"
    }

    fn title(&self) -> &'static str {
        "{project-name} {version} ({arch})"
    }

    fn string_name(&self) -> &'static str {
        "version"
    }

    impl_module!(
        project_name, version, version_tweak, build_type,
        sysname, arch, cmake_built_type, compile_time,
        compiler, libc, package_manager, release_link
    );
}

impl_display_for_module!(Version);