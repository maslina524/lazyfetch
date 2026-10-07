use crate::{
    detect::wallpaper::WallpaperInfo, 
    imp::error::ErrorCode, 
    warning, 
    windows::{
        encoding::{self, Utf16Len, utf16le_to_utf8}, 
        link::SystemParametersInfoW, 
        path::Path
    }
};

const MAX_PATH: usize = 260 + 1; // `+1` for `\0`
const SPI_GETDESKWALLPAPER: u32 = 0x0073;

impl WallpaperInfo {
    pub fn new() -> Self {
        let full_path = match Self::full_path() {
            Ok(p) => p,
            Err(e) => {
                warning!("Failed to get full wallpaper path: {e}");
                Path::new()
            }
        };

        Self {
            full_path
        }
    }
    
    fn full_path() -> encoding::Result<Path> {
        let mut buf = [0u16; MAX_PATH];

        // SAFETY: Completely safe
        let ret = unsafe {
            SystemParametersInfoW(
                SPI_GETDESKWALLPAPER, 
                MAX_PATH as u32, 
                (&raw mut buf).cast(), 
                0
            )
        };
        if ret == 0 {
            return Err(ErrorCode::last().into());
        }

        let utf8 = utf16le_to_utf8(&buf, Utf16Len::NullTerminated)?;
        Ok(Path::from(utf8))
    }
}