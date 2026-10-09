use alloc::string::String;
use doc::Docs;

use alloc::borrow::Cow;

use crate::{
    detect::os::OsInfo,
    format, impl_display_for_module, impl_module,
    logo::LogoInfo,
    modules::Module,
    str::{ConcatStr, SmolStr},
    sync::OnceLock,
};

static OS: OnceLock<Os> = OnceLock::new();

#[derive(Debug, Docs)]
pub struct Os {
    #[doc = "Name of the kernel"]
    pub sysname: &'static str,
    #[doc = "Name"]
    pub name: Cow<'static, str>,
    #[doc = "Pretty name, if available"]
    pub pretty_name: String,
    #[doc = "ID"]
    pub id: ConcatStr<2>,
    #[doc = "ID like"]
    pub id_like: ConcatStr<2>,
    #[doc = "Variant"]
    pub variant: Cow<'static, str>,
    #[doc = "Variant ID"]
    pub variant_id: Cow<'static, str>,
    #[doc = "Version"]
    pub version: SmolStr,
    #[doc = "Version ID"]
    pub version_id: SmolStr,
    #[doc = "Version codename"]
    pub codename: Cow<'static, str>,
    #[doc = "Build ID"]
    pub build_id: String,
    #[doc = "Architecture"]
    pub arch: &'static str,
    #[doc = "Logo as a nerd emoji"]
    pub nerd_emoji: char,
    #[doc = "Logo as a colored nerd emoji"]
    pub colored_nerd_emoji: String,
}

impl Module for Os {
    fn new() -> Self {
        let info = OsInfo::new();

        // #[cfg(target_os = "windows")]
        // let pretty_name = format!("{} {} ({})", info.id, info.variant, info.codename);
        // #[cfg(target_family = "unix")]
        let pretty_name = format!("{} {}", info.name, info.version);

        Self {
            sysname: info.sysname,
            name: info.name,
            pretty_name,
            id: info.id,
            id_like: info.id_like,
            variant: info.variant,
            variant_id: info.variant_id,
            version: info.version,
            version_id: info.version_id,
            codename: info.codename,
            build_id: String::new(),
            arch: env!("TARGET_ARCH"),
            nerd_emoji: info.nerd,
            colored_nerd_emoji: format!(
                "\x1b[{}m{}\x1b[0m",
                LogoInfo::get().expect("Unreachable").color_keys,
                info.nerd
            ),
        }
    }

    fn get() -> &'static Self {
        OS.get_or_init(Self::new)
    }

    fn key(&self) -> &'static str {
        "OS"
    }

    fn title(&self) -> &'static str {
        "{pretty-name} {arch}"
    }

    fn string_name(&self) -> &'static str {
        "os"
    }

    impl_module!(
        sysname,
        name,
        pretty_name,
        id,
        id_like,
        variant,
        variant_id,
        version,
        version_id,
        codename,
        build_id,
        arch,
        nerd_emoji
    );
}

impl_display_for_module!(Os);
