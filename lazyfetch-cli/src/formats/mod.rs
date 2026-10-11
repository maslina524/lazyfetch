#![doc = include_str!("README.md")]

use core::fmt::Write;

use alloc::{borrow::ToOwned, string::String, vec::Vec};

use crate::color;

mod display;
mod freq;
mod march;
mod mem;
mod percent;
mod splitted;
mod temp;
mod time;

pub use display::ZeroPaddedTwo;
pub use freq::Frequency;
pub use march::MicroArch;
pub use mem::MemorySize;
pub use percent::Percent;
pub use splitted::SplittedAnsiIter;
pub use temp::Temperature;
pub use time::Time;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ColorPlan {
    FG,
    BG,
}

pub struct StringFormatter<'a>(&'a mut String);

impl Write for StringFormatter<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.0.write_str(s)
    }
}

impl<'a> StringFormatter<'a> {
    pub const fn new(ptr: &'a mut String) -> Self {
        Self(ptr)
    }

    pub fn write_fmt(&mut self, args: core::fmt::Arguments) -> core::fmt::Result {
        core::fmt::Write::write_fmt(self, args)
    }

    pub fn write_nl(&mut self) -> core::fmt::Result {
        self.0.write_str("\n")
    }
}

pub fn char_width(c: char) -> usize {
    let cp = c as u32;

    if cp == 0
        || cp < 0x20
        || (0x7F..0xA0).contains(&cp)
        || (0x200B..=0x200F).contains(&cp)
        || (0x2028..=0x202E).contains(&cp)
        || (0x2060..=0x206F).contains(&cp)
        || cp == 0xFEFF
    {
        return 0;
    }

    if (0x0300..=0x036F).contains(&cp)
        || (0x1AB0..=0x1AFF).contains(&cp)
        || (0x1DC0..=0x1DFF).contains(&cp)
        || (0x20D0..=0x20FF).contains(&cp)
        || (0xFE20..=0xFE2F).contains(&cp)
    {
        return 0;
    }

    if is_wide(cp) {
        return 2;
    }

    1
}

const fn is_wide(cp: u32) -> bool {
    matches!(cp,
        0x1100..=0x115F   // Hangul Jamo
        | 0x2E80..=0x303E // CJK Radicals, Kangxi, CJK Symbols
        | 0x3041..=0x33FF // Hiragana, Katakana, Bopomofo, CJK Compat
        | 0x3400..=0x4DBF // CJK Ext A
        | 0x4E00..=0x9FFF // CJK Unified
        | 0xA000..=0xA4CF // Yi
        | 0xAC00..=0xD7A3 // Hangul Syllables
        | 0xF900..=0xFAFF // CJK Compat Ideographs
        | 0xFE10..=0xFE19 // Vertical forms
        | 0xFE30..=0xFE6F // CJK Compat Forms
        | 0xFF00..=0xFF60 // Fullwidth Forms
        | 0xFFE0..=0xFFE6 // Fullwidth signs
        | 0x1F300..=0x1F64F // Emoji
        | 0x1F900..=0x1F9FF
        | 0x20000..=0x2FFFD
        | 0x30000..=0x3FFFD
    )
}

pub fn visible_len(s: &str) -> usize {
    let mut count = 0;
    let mut skip = false;
    for ch in s.chars() {
        if skip {
            if ch == 'm' {
                skip = false;
            }
            continue;
        }
        if ch == '\x1b' {
            skip = true;
            continue;
        }
        count += char_width(ch);
    }
    count
}

pub fn expand_unicode(s: &str) -> String {
    let mut ret = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let mut pos = 0;

    while pos < chars.len() {
        if pos + 6 <= chars.len() && chars[pos] == '\\' && chars[pos + 1] == 'u' {
            let hex: String = chars[pos + 2..pos + 6].iter().collect();
            if let Ok(num) = u32::from_str_radix(&hex, 16) {
                if let Some(ch) = char::from_u32(num) {
                    ret.push(ch);
                }
                pos += 6;
                continue;
            }
        }
        ret.push(chars[pos]);
        pos += 1;
    }
    ret
}

pub fn expand_rust_unicode(s: &str) -> String {
    let mut ret = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let mut pos = 0;

    while pos < chars.len() {
        if pos + 3 <= chars.len()
            && chars[pos] == '\\'
            && chars[pos + 1] == 'u'
            && chars[pos + 2] == '{'
        {
            pos += 3;
            let mut hex = String::with_capacity(8);
            while pos < chars.len() && chars[pos] != '}' {
                hex.push(chars[pos]);
                pos += 1;
            }
            pos += 1;

            if let Ok(num) = u32::from_str_radix(&hex, 16)
                && let Some(ch) = char::from_u32(num)
            {
                ret.push(ch);
            }
            continue;
        }
        ret.push(chars[pos]);
        pos += 1;
    }
    ret
}

macro_rules! add_prefix {
    ($prefixes:expr, $ret:expr, $lit:literal, $constant:expr) => {{
        if $prefixes.contains(&$lit) {
            $ret.push($constant);
        }
    }};
}

fn is_ansi_color(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_digit() || c == ';')
}

pub fn format_color(s: &str, plan: ColorPlan) -> String {
    if is_ansi_color(s) {
        return s.to_owned();
    }

    let count = s.matches('_').count();
    let mut ret = Vec::with_capacity(count + 1);

    let (color, prefixes) = if count == 0 {
        (s, Vec::new())
    } else {
        let mut parts: Vec<&str> = s.split('_').collect();
        let color = parts.pop().unwrap();
        (color, parts)
    };

    // Supported named prefixes:
    // reset_, bright_, dim_, italic_, underline_,
    // blink_, inverse_, hidden_, strike_, light_
    if !prefixes.is_empty() {
        add_prefix!(prefixes, ret, "reset", color::MODE_RESET);
        add_prefix!(prefixes, ret, "bold", color::MODE_BOLD);
        add_prefix!(prefixes, ret, "dim", color::MODE_DIM);
        add_prefix!(prefixes, ret, "italic", color::MODE_ITALIC);
        add_prefix!(prefixes, ret, "underline", color::MODE_UNDERLINE);
        add_prefix!(prefixes, ret, "blink", color::MODE_BLINK);
        add_prefix!(prefixes, ret, "inverse", color::MODE_INVERSE);
        add_prefix!(prefixes, ret, "hidden", color::MODE_HIDDEN);
        add_prefix!(prefixes, ret, "strike", color::MODE_STRIKETHROUGH);
    }

    let is_light = prefixes.contains(&"light");
    let color_str = match (color, is_light, plan) {
        // Black
        ("black", false, ColorPlan::FG) => color::FG_BLACK,
        ("black", true, ColorPlan::FG) => color::FG_LIGHT_BLACK,
        ("black", false, ColorPlan::BG) => color::BG_BLACK,
        ("black", true, ColorPlan::BG) => color::BG_LIGHT_BLACK,
        // Red
        ("red", false, ColorPlan::FG) => color::FG_RED,
        ("red", true, ColorPlan::FG) => color::FG_LIGHT_RED,
        ("red", false, ColorPlan::BG) => color::BG_RED,
        ("red", true, ColorPlan::BG) => color::BG_LIGHT_RED,
        // Green
        ("green", false, ColorPlan::FG) => color::FG_GREEN,
        ("green", true, ColorPlan::FG) => color::FG_LIGHT_GREEN,
        ("green", false, ColorPlan::BG) => color::BG_GREEN,
        ("green", true, ColorPlan::BG) => color::BG_LIGHT_GREEN,
        // Yellow
        ("yellow", false, ColorPlan::FG) => color::FG_YELLOW,
        ("yellow", true, ColorPlan::FG) => color::FG_LIGHT_YELLOW,
        ("yellow", false, ColorPlan::BG) => color::BG_YELLOW,
        ("yellow", true, ColorPlan::BG) => color::BG_LIGHT_YELLOW,
        // Blue
        ("blue", false, ColorPlan::FG) => color::FG_BLUE,
        ("blue", true, ColorPlan::FG) => color::FG_LIGHT_BLUE,
        ("blue", false, ColorPlan::BG) => color::BG_BLUE,
        ("blue", true, ColorPlan::BG) => color::BG_LIGHT_BLUE,
        // Magenta
        ("magenta", false, ColorPlan::FG) => color::FG_MAGENTA,
        ("magenta", true, ColorPlan::FG) => color::FG_LIGHT_MAGENTA,
        ("magenta", false, ColorPlan::BG) => color::BG_MAGENTA,
        ("magenta", true, ColorPlan::BG) => color::BG_LIGHT_MAGENTA,
        // Cyan
        ("cyan", false, ColorPlan::FG) => color::FG_CYAN,
        ("cyan", true, ColorPlan::FG) => color::FG_LIGHT_CYAN,
        ("cyan", false, ColorPlan::BG) => color::BG_CYAN,
        ("cyan", true, ColorPlan::BG) => color::BG_LIGHT_CYAN,
        // White
        ("white", false, ColorPlan::FG) => color::FG_WHITE,
        ("white", true, ColorPlan::FG) => color::FG_LIGHT_WHITE,
        ("white", false, ColorPlan::BG) => color::BG_WHITE,
        ("white", true, ColorPlan::BG) => color::BG_LIGHT_WHITE,
        // Unknown color -> default
        _ => {
            if plan == ColorPlan::BG {
                color::BG_DEFAULT
            } else {
                color::FG_DEFAULT
            }
        }
    };
    ret.push(color_str);

    ret.join(";")
}

pub fn snake_to_camel_ascii(s: &str) -> String {
    let mut ret = String::with_capacity(s.len());
    let chars = s.chars();
    let mut transition = false;

    for ch in chars {
        if ch == '_' || ch == '-' {
            transition = true;
            continue;
        }
        if transition {
            if ch.is_ascii_lowercase() {
                let idx = ch as u32 - 32;
                ret.push(char::from_u32(idx).unwrap());
            } else {
                ret.push(ch);
            }

            transition = false;
            continue;
        }

        ret.push(ch);
    }

    ret
}

#[cfg(test)]
mod tests {
    use crate::formats::{MemorySize, SplittedAnsiIter, expand_rust_unicode, expand_unicode};

    #[test]
    fn test_conversion() {
        println!(
            "{}",
            expand_unicode(r"\u001b[31m \u001b[32m \u001b[33m \u001b[34m \u001b[0m")
        );
    }

    #[test]
    fn expand_rust_test() {
        println!("{}", expand_rust_unicode(r"\u{1b}[33mHello\u{1b}[0m"));
    }

    #[test]
    fn from_bytes_test() {
        assert_eq!(MemorySize::from_bytes(512).to_string(), "512 Bytes");
        assert_eq!(MemorySize::from_bytes(1024).to_string(), "1.00 Kb");
        assert_eq!(MemorySize::from_bytes(1536).to_string(), "1.50 Kb");
        assert_eq!(MemorySize::from_bytes(536_870_912).to_string(), "512.00 Mb");
        assert_eq!(MemorySize::from_bytes(2_147_483_648).to_string(), "2.00 Gb");
    }

    #[test]
    fn split_by_len_test() {
        let s = "HelloHelloHello";
        let lines = SplittedAnsiIter::new(s, 5).collect::<Vec<&str>>();
        assert_eq!(lines, vec!["Hello", "Hello", "Hello"]);
    }

    #[test]
    fn split_by_len_ansi_test() {
        let s = "\x1b[31mHelloHelloHello\x1b[0m";
        let lines = SplittedAnsiIter::new(s, 5).collect::<Vec<&str>>();
        assert_eq!(
            lines,
            vec![
                "\x1b[31mHello\x1b[0m",
                "\x1b[31mHello\x1b[0m",
                "\x1b[31mHello\x1b[0m"
            ]
        );
    }

    #[test]
    fn split_by_len_ansi_newline_test() {
        let s = "\n";
        let lines = SplittedAnsiIter::new(s, 999).collect::<Vec<&str>>();
        assert_ne!(lines, [] as [&str; 0]);
    }
}
