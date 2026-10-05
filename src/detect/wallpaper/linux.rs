use core::ffi::{CStr, c_char};

use alloc::{
    string::String,
    borrow::ToOwned
};

use crate::{
    detect::wallpaper::WallpaperInfo, 
    unix::{
        libc::{fgets, getenv, pclose, popen}, 
        path::Path
    }
};

impl WallpaperInfo {
    #[allow(clippy::option_if_let_else)]
    pub fn new() -> Self {
        let full_path = Self::xdg()
            .and_then(|s| Self::call(&s.trim().to_lowercase()))
            .map_or_default(|s| {
                Path::from(s.trim().trim_start_matches("file://"))
            });

        Self {
            full_path,
        }
    }

    fn cmd(xdg: &str) -> Option<&'static CStr> {
        match xdg {
            "gnome" | "cinnamon" | "unity" => Some(cr"gsettings get org.gnome.desktop.background picture-uri | sed -e 's/^'\''file:\/\///' -e 's/'\''$//'"),
            "mate" => Some(cr"gsettings get org.mate.background picture-filename | sed -e 's/^'\''//' -e 's/'\''$//'"),
            "kde" | "plasma" => Some(cr"kreadconfig5 --file ~/.config/plasma-org.kde.plasma.desktop-appletsrc --group Containments --key wallpaper | sed 's/^file:\/\///'"),
            "xfce" => Some(c"xfconf-query -c xfce4-desktop -p /backdrop/screen0/monitor0/image-path"),
            "lxde" => Some(cr"grep -oP 'wallpaper=\K.*' ~/.config/lxsession/LXDE/desktop.conf | head -1"),
            "lxqt" => Some(cr"grep -oP 'wallpaper=\K.*' ~/.config/lxqt/session.conf | head -1"),
            "deepin" => Some(cr"dconf read /com/deepin/dde/appearance/background-uris | sed -e 's/^\[//' -e 's/\]$//' -e 's/^'\''file:\/\///' -e 's/'\''$//'"),
            "pantheon" => Some(cr"gsettings get org.pantheon.desktop.gala.background picture-uri | sed -e 's/^'\''file:\/\///' -e 's/'\''$//'"),
            _ => None,
        }
    }

    fn call(xdg: &str) -> Option<String> {
        let cmd = Self::cmd(xdg)?;
        let typ = c"r";
        let fp = popen(cmd.as_ptr(), typ.as_ptr());

        let mut buf = [c_char::default(); 128 + 1];
        if fp.is_null() {
            return None;
        }
        loop {
            let ret = fgets(buf.as_mut_ptr(), 128, fp);
            if ret.is_null() {
                break;
            }
        }
        pclose(fp);

        // SAFETY: Libc is guaranteed to return a valid string
        let c_str = unsafe { CStr::from_ptr(buf.as_ptr().cast()) };
        let string = c_str.to_string_lossy().into_owned();
        Some(string)
    }

    fn xdg() -> Option<String> {
        let ptr = getenv(c"XDG_CURRENT_DESKTOP".as_ptr());
        if ptr.is_null() {
            return None;
        }
        // SAFETY: Libc is guaranteed to return a valid string
        let c_str = unsafe { CStr::from_ptr(ptr) };
        let string = c_str.to_string_lossy().trim().to_owned();
        
        if string.is_empty() {
            None
        } else {
            Some(string)
        }
    }
}