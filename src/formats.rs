use core::{cmp::Ordering, fmt::Write, ops::Range, str::FromStr};

use alloc::{
    borrow::{Cow, ToOwned},
    string::String,
    vec::Vec,
};

// Why does clippy think this variant is better than `colors::*`?
use crate::{
    color::{
        BG_BLACK, BG_BLUE, BG_CYAN, BG_DEFAULT, BG_GREEN, BG_LIGHT_BLACK, BG_LIGHT_BLUE,
        BG_LIGHT_CYAN, BG_LIGHT_GREEN, BG_LIGHT_MAGENTA, BG_LIGHT_RED, BG_LIGHT_WHITE,
        BG_LIGHT_YELLOW, BG_MAGENTA, BG_RED, BG_WHITE, BG_YELLOW, FG_BLACK, FG_BLUE, FG_CYAN,
        FG_DEFAULT, FG_GREEN, FG_LIGHT_BLACK, FG_LIGHT_BLUE, FG_LIGHT_CYAN, FG_LIGHT_GREEN,
        FG_LIGHT_MAGENTA, FG_LIGHT_RED, FG_LIGHT_WHITE, FG_LIGHT_YELLOW, FG_MAGENTA, FG_RED,
        FG_WHITE, FG_YELLOW, MODE_BLINK, MODE_BOLD, MODE_DIM, MODE_HIDDEN, MODE_INVERSE,
        MODE_ITALIC, MODE_RESET, MODE_STRIKETHROUGH, MODE_UNDERLINE,
    },
    config::Config,
    imp,
};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ColorPlan {
    FG,
    BG,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Frequency {
    Hz(u16),
    MHz(f32),
    GHz(f32),
}

impl Frequency {
    pub fn from_hz(hz: u64) -> Self {
        let mut divisions = 0;
        #[allow(
            clippy::cast_precision_loss,
            reason = "Mantissa is 52 bits, 2^52 = a lot"
        )]
        let mut f_hz = hz as f64;

        while f_hz >= 1000.0 && divisions < 2 {
            f_hz /= 1000.0;
            divisions += 1;
        }

        match divisions {
            0 => Self::Hz(hz as u16),
            1 => Self::MHz(f_hz as f32),
            2 => Self::GHz(f_hz as f32),
            _ => unreachable!(),
        }
    }

    pub fn as_hz(self) -> u64 {
        match self {
            Self::Hz(h) => h as u64,
            Self::MHz(h) => (h * 1000.0) as u64,
            Self::GHz(h) => (h * 1000.0 * 1000.0) as u64,
        }
    }
}

impl From<Frequency> for f64 {
    #[allow(clippy::cast_precision_loss)]
    fn from(val: Frequency) -> Self {
        val.as_hz() as Self
    }
}

impl Default for Frequency {
    fn default() -> Self {
        Self::Hz(0)
    }
}

impl core::fmt::Display for Frequency {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Hz(b) => write!(f, "{b} Hz"),
            Self::MHz(b) => write!(f, "{b:.02} MHz"),
            Self::GHz(b) => write!(f, "{b:.02} GHz"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemorySize {
    Byte(u16),
    Kb(f32),
    Mb(f32),
    Gb(f32),
}

impl MemorySize {
    pub fn from_bytes(bytes: u64) -> Self {
        let mut divisions = 0;
        #[allow(
            clippy::cast_precision_loss,
            reason = "Mantissa is 52 bits, 2^52 = 4 petabytes"
        )]
        let mut f_bytes = bytes as f64;

        while f_bytes >= 1024.0 && divisions < 3 {
            f_bytes /= 1024.0;
            divisions += 1;
        }

        match divisions {
            0 => Self::Byte(bytes as u16),
            1 => Self::Kb(f_bytes as f32),
            2 => Self::Mb(f_bytes as f32),
            3 => Self::Gb(f_bytes as f32),
            _ => unreachable!(),
        }
    }

    pub fn from_kilobytes(mut kilobytes: f64) -> Self {
        let mut divisions = 1;

        while kilobytes >= 1024.0 && divisions < 3 {
            kilobytes /= 1024.0;
            divisions += 1;
        }

        match divisions {
            1 => Self::Kb(kilobytes as f32),
            2 => Self::Mb(kilobytes as f32),
            3 => Self::Gb(kilobytes as f32),
            _ => unreachable!(),
        }
    }

    pub fn as_bytes(self) -> u64 {
        match self {
            Self::Byte(b) => b as u64,
            Self::Kb(b) => (b * 1024.0) as u64,
            Self::Mb(b) => (b * 1024.0 * 1024.0) as u64,
            Self::Gb(b) => (b * 1024.0 * 1024.0 * 1024.0) as u64,
        }
    }

    pub fn as_kilobytes(self) -> f64 {
        match self {
            Self::Byte(b) => b as f64 / 1024.0,
            Self::Kb(b) => b as f64,
            Self::Mb(b) => (b * 1024.0) as f64,
            Self::Gb(b) => (b * 1024.0 * 1024.0) as f64,
        }
    }
}

impl From<MemorySize> for f64 {
    #[allow(clippy::cast_precision_loss)]
    fn from(val: MemorySize) -> Self {
        val.as_bytes() as Self
    }
}

impl Default for MemorySize {
    fn default() -> Self {
        Self::Byte(0)
    }
}

impl core::fmt::Display for MemorySize {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Byte(b) => write!(f, "{b} Bytes"),
            Self::Kb(b) => write!(f, "{b:.02} Kb"),
            Self::Mb(b) => write!(f, "{b:.02} Mb"),
            Self::Gb(b) => write!(f, "{b:.02} Gb"),
        }
    }
}

impl PartialOrd for MemorySize {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_bytes().partial_cmp(&other.as_bytes())
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Percent(u8);

impl Percent {
    pub const fn get(self) -> u8 {
        self.0
    }

    pub const fn new(mut percent: u8) -> Self {
        if percent > 100 {
            percent = 100;
        }
        Self(percent)
    }

    pub const fn new_check(percent: u8) -> Option<Self> {
        if percent > 100 {
            None
        } else {
            Some(Self(percent))
        }
    }
}

impl FromStr for Percent {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.trim()
            .trim_end_matches('%')
            .parse::<u8>()
            .map_err(|_| ())
            .and_then(|v| Self::new_check(v).ok_or(()))
    }
}

impl From<Percent> for f64 {
    #[allow(clippy::cast_precision_loss)]
    fn from(val: Percent) -> Self {
        val.get() as Self
    }
}

impl core::fmt::Display for Percent {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let val = Config::get().format_percent(*self);
        write!(f, "{val}")
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Temperature {
    Celsius(f32),
    Fahrenheit(f32),
    Kelvin(f32),
}

impl Temperature {
    pub const fn get(self) -> f32 {
        match self {
            Self::Celsius(t) | Self::Fahrenheit(t) | Self::Kelvin(t) => t,
        }
    }

    pub const fn symbol(self) -> char {
        match self {
            Self::Celsius(_) => 'C',
            Self::Fahrenheit(_) => 'F',
            Self::Kelvin(_) => 'K',
        }
    }

    pub const fn as_celsius(self) -> Self {
        let temp = match self {
            Self::Celsius(t) => t,
            Self::Fahrenheit(t) => (t - 32.0) * 5.0 / 9.0,
            Self::Kelvin(t) => t - 273.15,
        };
        Self::Celsius(temp)
    }

    pub const fn as_fahrenheit(self) -> Self {
        let temp = match self {
            Self::Celsius(t) => (t * 9.0 / 5.0) + 32.0,
            Self::Fahrenheit(t) => t,
            Self::Kelvin(t) => (t - 273.15) * 9.0 / 5.0 + 32.0,
        };
        Self::Fahrenheit(temp)
    }

    pub const fn as_kelvin(self) -> Self {
        let temp = match self {
            Self::Celsius(t) => t + 273.15,
            Self::Fahrenheit(t) => (t - 32.0) * 5.0 / 9.0 + 273.15,
            Self::Kelvin(t) => t,
        };
        Self::Kelvin(temp)
    }
}

impl Default for Temperature {
    fn default() -> Self {
        Self::Celsius(0.0)
    }
}

impl FromStr for Temperature {
    type Err = ();
    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        s = s.trim().trim_start_matches('+');
        let (val_str, suffix) = if s.ends_with("°C") {
            (s.trim_end_matches("°C"), "°C")
        } else if s.ends_with("°F") {
            (s.trim_end_matches("°F"), "°F")
        } else if s.ends_with("°K") {
            (s.trim_end_matches("°K"), "°K")
        } else {
            return Err(());
        };
        let val = val_str.parse::<f32>().map_err(|_| ())?;
        match suffix {
            "°C" => Ok(Self::Celsius(val)),
            "°F" => Ok(Self::Fahrenheit(val)),
            "°K" => Ok(Self::Kelvin(val)),
            _ => unreachable!(),
        }
    }
}

impl From<Temperature> for f64 {
    #[allow(clippy::cast_precision_loss)]
    fn from(val: Temperature) -> Self {
        val.get() as Self
    }
}

impl core::fmt::Display for Temperature {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let val = Config::get().format_temperature(*self);
        write!(f, "{val}")
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Time(u64);

impl Time {
    pub const fn new(time: u64) -> Self {
        Self(time)
    }
}

impl From<Time> for f64 {
    #[allow(clippy::cast_precision_loss)]
    fn from(val: Time) -> Self {
        val.0 as Self
    }
}

impl core::fmt::Display for Time {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let string = imp::env::format_timestamp(self.0, None);
        write!(f, "{string}")
    }
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
        add_prefix!(prefixes, ret, "reset", MODE_RESET);
        add_prefix!(prefixes, ret, "bold", MODE_BOLD);
        add_prefix!(prefixes, ret, "dim", MODE_DIM);
        add_prefix!(prefixes, ret, "italic", MODE_ITALIC);
        add_prefix!(prefixes, ret, "underline", MODE_UNDERLINE);
        add_prefix!(prefixes, ret, "blink", MODE_BLINK);
        add_prefix!(prefixes, ret, "inverse", MODE_INVERSE);
        add_prefix!(prefixes, ret, "hidden", MODE_HIDDEN);
        add_prefix!(prefixes, ret, "strike", MODE_STRIKETHROUGH);
    }

    let is_light = prefixes.contains(&"light");
    let color_str = match (color, is_light, plan) {
        // Black
        ("black", false, ColorPlan::FG) => FG_BLACK,
        ("black", true, ColorPlan::FG) => FG_LIGHT_BLACK,
        ("black", false, ColorPlan::BG) => BG_BLACK,
        ("black", true, ColorPlan::BG) => BG_LIGHT_BLACK,
        // Red
        ("red", false, ColorPlan::FG) => FG_RED,
        ("red", true, ColorPlan::FG) => FG_LIGHT_RED,
        ("red", false, ColorPlan::BG) => BG_RED,
        ("red", true, ColorPlan::BG) => BG_LIGHT_RED,
        // Green
        ("green", false, ColorPlan::FG) => FG_GREEN,
        ("green", true, ColorPlan::FG) => FG_LIGHT_GREEN,
        ("green", false, ColorPlan::BG) => BG_GREEN,
        ("green", true, ColorPlan::BG) => BG_LIGHT_GREEN,
        // Yellow
        ("yellow", false, ColorPlan::FG) => FG_YELLOW,
        ("yellow", true, ColorPlan::FG) => FG_LIGHT_YELLOW,
        ("yellow", false, ColorPlan::BG) => BG_YELLOW,
        ("yellow", true, ColorPlan::BG) => BG_LIGHT_YELLOW,
        // Blue
        ("blue", false, ColorPlan::FG) => FG_BLUE,
        ("blue", true, ColorPlan::FG) => FG_LIGHT_BLUE,
        ("blue", false, ColorPlan::BG) => BG_BLUE,
        ("blue", true, ColorPlan::BG) => BG_LIGHT_BLUE,
        // Magenta
        ("magenta", false, ColorPlan::FG) => FG_MAGENTA,
        ("magenta", true, ColorPlan::FG) => FG_LIGHT_MAGENTA,
        ("magenta", false, ColorPlan::BG) => BG_MAGENTA,
        ("magenta", true, ColorPlan::BG) => BG_LIGHT_MAGENTA,
        // Cyan
        ("cyan", false, ColorPlan::FG) => FG_CYAN,
        ("cyan", true, ColorPlan::FG) => FG_LIGHT_CYAN,
        ("cyan", false, ColorPlan::BG) => BG_CYAN,
        ("cyan", true, ColorPlan::BG) => BG_LIGHT_CYAN,
        // White
        ("white", false, ColorPlan::FG) => FG_WHITE,
        ("white", true, ColorPlan::FG) => FG_LIGHT_WHITE,
        ("white", false, ColorPlan::BG) => BG_WHITE,
        ("white", true, ColorPlan::BG) => BG_LIGHT_WHITE,
        // Unknown color -> default
        _ => {
            if plan == ColorPlan::BG {
                BG_DEFAULT
            } else {
                FG_DEFAULT
            }
        }
    };
    ret.push(color_str);

    ret.join(";")
}

#[derive(Clone)]
pub struct SplittedAnsiIter {
    buf: &'static str,
    ranges: Vec<Range<usize>>,
    index: usize,
}

impl SplittedAnsiIter {
    pub const fn empty() -> Self {
        Self {
            buf: "",
            ranges: Vec::new(),
            index: 0,
        }
    }

    pub fn new(s: &str, len: usize) -> Self {
        if s.is_empty() {
            return Self::empty();
        }

        let mut buf = String::with_capacity(s.len() + 32);
        let mut ranges: Vec<Range<usize>> = Vec::with_capacity(8); // Random cap value

        let mut build_len = 0;
        let mut chunk_start = 0;
        let mut current_ansi = String::from("\x1b[0m");
        let mut ansi_is_default = true;

        let mut it = s.chars();
        while let Some(ch) = it.next() {
            if ch == '\x1b' {
                current_ansi.clear();
                current_ansi.push('\x1b');
                buf.push('\x1b');
                for c in it.by_ref() {
                    current_ansi.push(c);
                    buf.push(c);
                    if c == 'm' { break; }
                }
                ansi_is_default = current_ansi == "\x1b[0m";
                continue;
            }

            if ch == '\n' {
                if !ansi_is_default { buf.push_str("\x1b[0m"); }

                if build_len > 0 {
                    ranges.push(chunk_start..buf.len());
                } else {
                    buf.truncate(chunk_start);
                }
                build_len = 0;
                chunk_start = buf.len();

                if !ansi_is_default { buf.push_str(&current_ansi); }
                continue;
            }

            buf.push(ch);
            build_len += 1;

            if build_len >= len {
                if !ansi_is_default { buf.push_str("\x1b[0m"); }

                ranges.push(chunk_start..buf.len());
                build_len = 0;
                chunk_start = buf.len();

                if !ansi_is_default { buf.push_str(&current_ansi); }
            }
        }

        if current_ansi != "\x1b[0m" {
            buf.push_str("\x1b[0m");
        }
        ranges.push(chunk_start..buf.len());

        Self {
            buf: String::leak(buf),
            ranges,
            index: 0,
        }
    }

    pub const fn len(&self) -> usize {
        self.ranges.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }
}

impl Iterator for SplittedAnsiIter {
    type Item = &'static str;

    fn next(&mut self) -> Option<&'static str> {
        let r = self.ranges.get(self.index)?;
        self.index += 1;
        Some(&self.buf[r.clone()])
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let rem = self.ranges.len() - self.index;
        (rem, Some(rem))
    }
}

impl ExactSizeIterator for SplittedAnsiIter {}

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

pub fn lazy_replace<'b>(
    s: &'b str,
    key: &'b str,
    value_fn: impl FnOnce() -> String,
) -> Cow<'b, str> {
    if s.contains(key) {
        Cow::Owned(s.replace(key, &value_fn()))
    } else {
        Cow::Borrowed(s)
    }
}

#[macro_export]
macro_rules! format {
    ($($tt:tt)*) => {{
        let mut string = alloc::string::String::with_capacity(16);
        let mut formatter = $crate::formats::StringFormatter::new(&mut string);
        let _ = formatter.write_fmt(format_args!($($tt)*));
        string
    }};
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
