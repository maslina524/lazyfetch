
use crate::{
    detect::wallpaper::WallpaperInfo, 
    unix::path::Path
};

impl WallpaperInfo {
    pub fn new() -> Self {
        Self {
            full_path: Path::new(),
        }
    }
}