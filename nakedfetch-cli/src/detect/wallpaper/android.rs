use crate::{detect::wallpaper::WallpaperInfo, unix::path::Path};

impl WallpaperInfo {
    pub const fn new() -> Self {
        Self {
            full_path: Path::new(),
        }
    }
}
