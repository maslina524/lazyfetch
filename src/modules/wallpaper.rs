use doc::Docs;

use crate::{
    impl_module,
    impl_display_for_module,
    modules::Module,
    sync::OnceLock,
    detect::wallpaper::WallpaperInfo,
    imp::path::Path,
    str::SmolStr
};

static WALLPAPER: OnceLock<Wallpaper> = OnceLock::new();

#[derive(Debug, Docs)]
pub struct Wallpaper {
    #[doc = "File name"]
    pub file_name: SmolStr,
    #[doc = "Full path"]
    pub full_path: Path
}

impl Module for Wallpaper {
    #[allow(clippy::redundant_closure)]
    fn new() -> Self {
        let info = WallpaperInfo::new();
        let file_name = info.full_path
            .last()
            .map_or_default(SmolStr::from);

        Self {
            file_name,
            full_path: info.full_path
        }
    }

    fn get() -> &'static Self {
        WALLPAPER.get_or_init(|| {
            Self::new()
        })
    }

    fn key(&self) -> &'static str {
        "Wallpaper"
    }

    fn title(&self) -> &'static str {
        "{file-name}"
    }

    fn string_name(&self) -> &'static str {
        "wallpaper"
    }

    impl_module!(
        file_name, full_path
    );
}

impl_display_for_module!(Wallpaper);