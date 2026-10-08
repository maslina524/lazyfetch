#![doc = include_str!("README.md")]

use core::fmt::Write;

use alloc::{
    string::String, 
    vec::Vec
};

use crate::{
    abort, 
    color, 
    formats::char_width, 
    image::Image, 
    sync::OnceLock, 
    zlib
};

macro_rules! logo_mod {
    ($($letter:ident),* $(,)?) => {
        $(
            include!(concat!(env!("OUT_DIR"), "/logo/", stringify!($letter), ".rs"));
        )*
    };
}

logo_mod!(
    a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x, y, z
);

const UNKNOWN: &[u8] = include_bytes!(concat!(env!("LOGO_OUT_DIR"), "/temp/_/unknown.txt"));
static UNKNOWN_PTR: &LogoInfo = &LogoInfo {
    names: &[],
    encoded: UNKNOWN,
    colors: &[],
    color_keys: color::FG_DEFAULT,
    color_title: color::FG_DEFAULT,
};

static LOGO_INFO: OnceLock<&'static LogoInfo> = OnceLock::new();

pub enum UILogo {
    None,
    Preset,
    Ascii(String),
    Image(Image),
}

pub struct LogoInfo {
    pub names: &'static [&'static str],
    pub encoded: &'static [u8],
    pub colors: &'static [&'static str],
    pub color_keys: &'static str,
    pub color_title: &'static str,
}

impl LogoInfo {
    pub fn new(name: &str) -> &'static Self {
        LOGO_INFO.get_or_init(|| {
            let Some(first_char) = name.chars().next() else {
                abort!("An empty string was passed for logo")
            };

            let stack = match first_char.to_ascii_lowercase() {
                'a' => A,
                'b' => B,
                'c' => C,
                'd' => D,
                'e' => E,
                'f' => F,
                'g' => G,
                'h' => H,
                'i' => I,
                'j' => J,
                'k' => K,
                'l' => L,
                'm' => M,
                'n' => N,
                'o' => O,
                'p' => P,
                'q' => Q,
                'r' => R,
                's' => S,
                't' => T,
                'u' => U,
                'v' => V,
                'w' => W,
                'x' => X,
                'y' => Y,
                'z' => Z,
                _ => return UNKNOWN_PTR,
            };

            for logo in stack {
                if logo.names.iter().any(|n| n.eq_ignore_ascii_case(name)) {
                    return logo;
                }
            }

            UNKNOWN_PTR
        })
    }

    pub fn get() -> Option<&'static Self> {
        LOGO_INFO.get().map(|v| &**v)
    }

    pub fn get_ready_logo_lines(&self, logo: UILogo) -> Vec<(String, usize)> {
        let lines_string = match logo {
            UILogo::Preset => {
                let mut decompressed = Vec::with_capacity(self.encoded.len());
                zlib::decompress(self.encoded.to_vec(), &mut decompressed);
                String::from_utf8(decompressed).expect("Non Utf8 in logo")
            }
            UILogo::Ascii(s) => s,
            _ => return Vec::new(),
        };

        let lines_count = lines_string.chars().filter(|c| *c == '\n').count() + 1;
        let mut ret = Vec::with_capacity(lines_count);
        let mut cur_code = self.colors.first().copied().unwrap_or("");

        for line in lines_string.lines() {
            let mut ret_len = 0usize;
            let mut ret_line = String::with_capacity(line.len() + 16);
            let _ = write!(ret_line, "\x1b[1;{cur_code}m");

            let mut in_percent = false;
            for ch in line.chars() {
                if ch == '$' {
                    if in_percent {
                        ret_line.push('$');
                        ret_len += 1;
                    }
                    in_percent = !in_percent;
                    continue;
                }
                if in_percent {
                    in_percent = false;
                    if let Some(i) = ch.to_digit(10)
                        && i > 0
                    {
                        let code = self.colors.get(i as usize - 1).copied().unwrap_or("0");
                        cur_code = code;
                        let _ = write!(ret_line, "\x1b[1;{cur_code}m");
                        continue;
                    }
                    let _ = write!(ret_line, "${ch}");
                    ret_len += 2;
                    continue;
                }
                ret_line.push(ch);
                ret_len += char_width(ch);
            }
            ret_line.push_str("\x1b[0m");
            ret.push((ret_line, ret_len));
        }
        ret
    }
}
