#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![deny(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![warn(clippy::missing_safety_doc, clippy::undocumented_unsafe_blocks)]
#![allow(
    dead_code,
    reason = "A lot of code is made \"for the future\"; all unused code will be removed by release"
)]
#![allow(
    clippy::too_many_lines,
    reason = "In logo/{a-z}.rs there are functions longer than 100 lines"
)]
#![allow(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "There's no point in them, it will make the code cleaner"
)]
#![allow(clippy::cargo_common_metadata)]

mod base64;
mod color;
mod config;
mod crc32;
mod deflate;
mod formats;
mod huffman;
mod image;
mod kitty;
mod superstr;
mod lua;
mod lz77;
mod macros;
mod nvidia;
mod png;
mod sync;
mod url;
mod zlib;

mod detect;
mod json;
mod logo;
mod modules;
mod ui;

cfg_if! {
    if #[cfg(target_os = "windows")] {
        mod windows;
        use windows as imp;
    } else if #[cfg(any(target_os = "linux", target_os = "android"))] {
        mod linux;
        use linux as imp;
    } else {
        compile_error!("Unsupported OS");
    }
}

extern crate alloc;

use core::{
    env, 
    ffi::c_int, 
    slice::Iter
};

use alloc::{
    string::String,
    boxed::Box,
    vec::Vec
};

use crate::{
    config::{Config, ConfigModule}, 
    formats::{SplittedAnsiIter, MemorySize}, 
    image::Image, 
    detect::os,
    imp::{
        allocator::{AllocationReport, Allocator}, 
        env, 
        fs, 
        http::Request
    }, 
    json::Json, 
    logo::{LogoInfo, UILogo},
    modules::{
        Commit, DocsVtable, FormatValue, Module, Version
    }, 
    nvidia::NvidiaLib, 
    png::Png, 
    sync::OnceLock, 
    url::Url,
};

#[global_allocator]
static ALLOCATOR: Allocator = Allocator;

static HELP_STRING: &str = include_str!(concat!(env!("OUT_DIR"), "/help.txt"));

const MIN_OFFSET: usize = 24;
const IMAGE_SIZE: usize = 40;
const CELL_ASPECT: f64 = 2.0;
const ALLOC_REP_BAR_SIZE: u128 = 64;

#[cfg(not(test))]
mod panic_impl {
    use core::panic::PanicInfo;

    use crate::{eprintln, exit, format};

    #[panic_handler]
    fn panic(info: &PanicInfo) -> ! {
        let msg = format!("{}", info.message());

        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()));

        if let Some(ref loc) = location {
            eprintln!("\x1b[1m{loc}: {msg}\x1b[0m");
        } else {
            eprintln!("\x1b[1m{msg}\x1b[0m");
        }

        let link = format!(
            "https://github.com/maslina524/lazyfetch/issues/new?template=panic.yaml&version={}&title={}&target={}&location={}",
            env!("CARGO_PKG_VERSION"),
            msg,
            env!("TARGET"),
            location.unwrap_or_default()
        );
        eprintln!("Leave an Issue at this [\x1b]8;;{link}\x1b\\link\x1b]8;;\x1b\\]");

        exit(101)
    }
}

fn max_line_len(lines: &[(String, usize)]) -> usize {
    let mut ret = 0;
    for (_, len) in lines {
        if *len > ret {
            ret = *len;
        }
    }
    ret
}

fn build_logo_buf(lines: &[(String, usize)], max_len: usize) -> Vec<String> {
    let padding = Config::get().get_logo_padding();
    let max_len_padding = max_len + padding.left + padding.right;
    let mut ret = Vec::with_capacity(lines.len() + padding.top + padding.bottom);

    let (w, _) = env::terminal_size();
    if w < max_len_padding {
        return Vec::new();
    }

    for _ in 0..padding.top {
        ret.push(" ".repeat(max_len_padding));
    }
    for (line, len) in lines {
        let string = format!(
            "{}{}{}",
            " ".repeat(padding.left),
            line,
            " ".repeat(padding.right + max_len - len)
        );
        ret.push(string);
    }
    for _ in 0..padding.bottom {
        ret.push(" ".repeat(max_len_padding));
    }

    ret
}

fn get_module_lines(preset_module: &ConfigModule, max_len_line: usize) -> Option<SplittedAnsiIter> {
    modules::from_preset_module(&preset_module.typ).map(|module| {
        let string = module.format(
            FormatValue {
                format: preset_module.key.as_deref(),
                color: preset_module.key_color.as_deref(),
            },
            FormatValue {
                format: preset_module.format.as_deref(),
                color: None,
            },
            Some(&preset_module.map),
        );
        string.map_or_else(SplittedAnsiIter::empty, |s| {
            SplittedAnsiIter::new(&s, max_len_line)
        })
    })
}

fn build_info_buf(max_len: usize) -> Vec<&'static str> {
    let padding = Config::get().get_logo_padding();
    let (w, _) = env::terminal_size();
    let max_len_line = if max_len + padding.left + padding.right < w {
        w - max_len - padding.left - padding.right
    } else {
        w
    };
    let mut ret = Vec::with_capacity(40);
    let preset = Config::get();

    for module in preset.modules() {
        if let Some(m) = get_module_lines(module, max_len_line) {
            ret.extend(m);
        }
    }

    ret
}

pub fn exit(code: u32) -> ! {
    cfg_if! {
        if #[cfg(any(target_os = "linux", target_os = "android"))] {
            // SAFETY: Run in binary, safe
            unsafe { crate::linux::libc::exit(code as i32) }
        } else if #[cfg(target_os = "windows")] {
            // SAFETY: Run in binary, safe
            unsafe { crate::windows::link::ExitProcess(code) }
        }
    }
}

fn get_config(args: &mut Iter<'_, String>) -> Config {
    let Some(_) = args.position(|a| a == "--config" || a == "-c") else {
        return Config::default();
    };
    let Some(path) = args.next() else {
        print_help(None);
    };

    // URL path
    if let Some(url) = Url::new(path) {
        let response = match Request::from_url(url).get() {
            Ok(r) => r,
            Err(e) => {
                warning!("Failed to connect to server (config): {}", e.code());
                return Config::default();
            }
        };
        if !response.is_success() {
            warning!("Failed to get preset from URL, Code: {}", response.code());
            return Config::default();
        }
        let Ok(t) = response.as_text() else {
            warning!("Failed to parse the response");
            return Config::default();
        };
        return match Json::from_str(&t) {
            Ok(c) => Config::from_json(&c),
            Err(e) => {
                warning!("Failed to parse the json config: {e}");
                Config::default()
            }
        };
    }

    // FS path
    match Json::from_file(path) {
        Ok(c) => Config::from_json(&c),
        Err(e) => {
            warning!("Failed to parse the json config: {e}");
            Config::default()
        }
    }
}

fn get_logo_name_and_custom(val: &str) -> (Box<str>, UILogo) {
    let id = os::get_id().as_boxed_str();
    match fs::read(val) {
        Ok(b) => {
            if png::is_png(&b) {
                match Png::decode(&b) {
                    Ok(p) => (id, UILogo::Image(p.into_image())),
                    Err(e) => {
                        warning!("Failed to parse png: {e}");
                        (id, UILogo::Preset)
                    }
                }
                
            } else if let Ok(s) = String::from_utf8(b) {
                (id, UILogo::Ascii(s))
            } else {
                warning!("Failed to represent data as utf8");
                (id, UILogo::Preset)
            }
        }
        Err(e) if e.is_file_not_found() => (
            val.to_lowercase().replace('_', " ").into_boxed_str(), 
            UILogo::Preset
        ),
        Err(e) => {
            warning!("Failed to use logo from fs: {e}");
            (id, UILogo::Preset)
        }
    }
}

#[cold]
fn print_help(theme: Option<&str>) -> ! {
    if let Some(t) = theme {
        let idx = t.find('-').unwrap_or_else(|| exit(1));
        let ident = &t[..idx];
        let action = &t[idx + 1..];

        let vtable = DocsVtable::from_str(ident).unwrap_or_else(|| exit(1));
        match action {
            "format" => {
                if let Some(doc) = (vtable.format)() {
                    println!(
                        "# In config file: {{ \"type\": \"{ident}\", \"format\": \"{{{}}} or {{1}}\" }}",
                        doc[0].name
                    );
                    println!("The following variables are passed:");
                    for i in doc {
                        println!(
                            "{:>24} : {:<4} : {}",
                            i.name,
                            i.second,
                            i.desc.unwrap_or("Empty")
                        );
                    }
                } else {
                    println!("Module `{ident}` doesn't support output formatting");
                }
            }
            "lua" => {
                if let Some(doc) = (vtable.lua)() {
                    println!(
                        "# In config file: {{ \"type\": \"{ident}\", \"format\": \"lua: return (...).{}\" }}",
                        doc[0].name
                    );
                    println!("The following variables are passed:");
                    for i in doc {
                        println!(
                            "{:>24} : {:<6} : {}",
                            i.name,
                            i.second,
                            i.desc.unwrap_or("Empty")
                        );
                    }
                } else {
                    println!("Module `{ident}` doesn't support lua");
                }
            }
            "example" => {
                let id = os::get_id().as_boxed_str();
                LogoInfo::new(&id);
                Config::get_or_init(Config::default());

                if let Some(doc) = (vtable.example)() {
                    // println!(
                    //     "# In config file: {{ \"type\": \"{ident}\", \"format\": \"lua: return (...).{}\" }}",
                    //     doc[0].name
                    // );
                    println!("The following variables are passed:");
                    for (name, value) in doc {
                        println!("{name:>24} : {value}");
                    }
                } else {
                    println!("Module `{ident}` doesn't have variables");
                }
            }
            _ => {
                println!("Incorrect action");
                exit(1);
            }
        }

        exit(0);
    }

    println!("{HELP_STRING}");

    exit(0)
}

#[cold]
fn print_version(method: Option<&str>) -> ! {
    let ver = Version::new();
    match method {
        None => println!("{} {} ({})", ver.project_name, ver.version, ver.arch),
        Some("raw") => println!("{}", ver.version),
        Some("dbg") => println!("{ver:#?}"),
        Some("hash") => println!("{}", ver.hash),
        Some("extended") => {
            let typ = match ver.build_type {
                "release" => "\x1b[32mrelease\x1b[0m",
                "debug" => "\x1b[33mdebug\x1b[0m",
                _ => unreachable!(),
            };
            let com = Commit::new();
            println!(
                "\x1b[1m{} {} ({}) {typ} [\x1b]8;;{}\x1b\\link\x1b]8;;\x1b\\]",
                ver.project_name, ver.version, ver.arch, ver.release_link
            );
            println!("    {}, {}", ver.compiler, ver.package_manager);
            println!(
                "    {} <{}> ({})",
                com.message, com.date_small, com.sha_short
            );
        }
        _ => eprintln!("Unknown method for version, supported: raw, dbg, extended"),
    }
    exit(0)
}

#[cold]
#[inline(never)]
fn print_alloc_report() {
    let rep = AllocationReport::get();
    let total = rep.alloc + rep.dealloc + rep.realloc;

    println!();
    println!("\x1b[1mAllocation Report\x1b[0m");
    println!("Total Ops: {total}");

    let total_u = total as u128;

    let mut alloc_len = ((rep.alloc as u128) * ALLOC_REP_BAR_SIZE) / total_u;
    let mut realloc_len = ((rep.realloc as u128) * ALLOC_REP_BAR_SIZE) / total_u;
    if alloc_len + realloc_len > ALLOC_REP_BAR_SIZE {
        alloc_len = ALLOC_REP_BAR_SIZE;
        realloc_len = 0;
    }
    let dealloc_len = ALLOC_REP_BAR_SIZE - alloc_len - realloc_len;

    let alloc_len = alloc_len as usize;
    let realloc_len = realloc_len as usize;
    let dealloc_len = dealloc_len as usize;

    println!(
        "|\x1b[{};1m{}\x1b[{};1m{}\x1b[{};1m{}\x1b[0m|",
        color::FG_YELLOW,        "=".repeat(alloc_len),
        color::FG_CYAN,          "=".repeat(dealloc_len),
        color::FG_LIGHT_MAGENTA, "=".repeat(realloc_len),
    );

    println!(
        "\x1b[{}mAlloc: {}   \x1b[{}mDealloc: {}   \x1b[{}mRealloc: {}\x1b[0m\n",
        color::FG_YELLOW,        rep.alloc,
        color::FG_CYAN,          rep.dealloc,
        color::FG_LIGHT_MAGENTA, rep.realloc,
    );

    println!("Total Bytes:");
    println!(
        "Allocated: {}   Deallocated: {}   Max In RT: {}", 
        MemorySize::from_bytes(rep.alloc_total), 
        MemorySize::from_bytes(rep.dealloc_total), 
        MemorySize::from_bytes(rep.max_in_runtime)
    );
}

fn print_none() {
    let id = os::get_id().as_boxed_str();
    LogoInfo::new(&id);

    let (w, _) = env::terminal_size();
    let info_lines = build_info_buf(w);

    for line in info_lines {
        println!("{line}\x1b[0m");
    }
}

#[allow(clippy::cast_precision_loss)]
fn print_image(image: &Image) {
    let id = os::get_id().as_boxed_str();
    LogoInfo::new(&id);

    let padding = Config::get().get_logo_padding();
    let (w, term_h) = env::terminal_size();

    let (img_w, img_h) = image.size();

    let mut cols = IMAGE_SIZE;
    let mut height = (img_h as f64 / img_w as f64 * cols as f64 / CELL_ASPECT) as usize;

    let max_h = term_h.saturating_sub(padding.top + padding.bottom).max(1);
    if height > max_h {
        let scale = max_h as f64 / height as f64;
        cols = ((cols as f64 * scale) as usize).max(1);
        height = max_h;
    }

    let max_logo_len_padding = cols + padding.left + padding.right;
    let empty_logo_line = " ".repeat(max_logo_len_padding);
    let info_buf = build_info_buf(w);
    let lines_printed;

    if max_logo_len_padding + MIN_OFFSET < w {
        lines_printed = height.max(info_buf.len());
        for i in 0..lines_printed {
            let info_line = info_buf.get(i).map_or("", |s| *s);
            println!("{empty_logo_line}{info_line}\x1b[0m");
        }
    } else {
        lines_printed = height + info_buf.len();
        for _ in 0..height {
            println!("{empty_logo_line}");
        }
        for line in info_buf {
            println!("{line}\x1b[0m");
        }
    }

    print!("\x1b7\x1b[{lines_printed}A\x1b[{}C", padding.left);

    crate::kitty::print_image(image, Some(cols), Some(height), 0);
    print!("\x1b8");
}

fn print_base(logo_name: &str) {
    let logo = LogoInfo::new(logo_name);
    just_print_logo_and_info(&logo.get_ready_logo_lines(UILogo::Preset));
}

fn print_ascii(ascii: String) {
    let id = os::get_id().as_boxed_str();
    let logo = LogoInfo::new(&id);
    just_print_logo_and_info(&logo.get_ready_logo_lines(UILogo::Ascii(ascii)));
}

fn just_print_logo_and_info(logo_lines: &[(String, usize)]) {
    let max_logo_len = max_line_len(logo_lines);
    let padding = Config::get().get_logo_padding();
    let max_logo_len_padding = max_logo_len + padding.left + padding.right;
    let (w, _) = env::terminal_size();
    let split_len = if max_logo_len_padding + MIN_OFFSET < w {
        max_logo_len
    } else {
        w
    };
    let logo_buf = build_logo_buf(logo_lines, max_logo_len);
    let info_buf = build_info_buf(split_len);
    let max_lines = logo_buf.len().max(info_buf.len());

    // Print buffers
    if max_logo_len_padding + MIN_OFFSET < w {
        let empty_logo_line = " ".repeat(max_logo_len_padding);
        for i in 0..max_lines {
            let logo_line = logo_buf
                .get(i)
                .map_or(empty_logo_line.as_str(), String::as_str);

            let info_line = info_buf.get(i).map_or("", |s| *s);
            println!("{logo_line}{info_line}\x1b[0m");
        }
    } else {
        for line in logo_buf {
            println!("{line}\x1b[0m");
        }
        for line in info_buf {
            println!("{line}\x1b[0m");
        }
    }
}

static ARGS: OnceLock<Vec<String>> = OnceLock::new();

#[cfg(any(target_os = "linux", target_os = "android"))]
use core::ffi::c_char;

/*
ANDROID BUILD:
rustup toolchain install nightly
rustup component add rust-src --toolchain nightly
rustup target add aarch64-linux-android --toolchain nightly

cd ~
wget https://dl.google.com/android/repository/android-ndk-r27c-linux.zip
unzip android-ndk-r27c-linux.zip
~/android-ndk-r27c/ndk-build --version

`bash`:
export ANDROID_NDK_HOME=$HOME/android-ndk-r27c
export ANDROID_NDK_ROOT=$ANDROID_NDK_HOME
export ANDROID_PLATFORM=24
export PATH=$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin:$PATH
cargo ndk -t arm64-v8a --platform 24 build --release

`fish`:
set -gx ANDROID_NDK_HOME $HOME/android-ndk-r27c
set -gx ANDROID_NDK_ROOT $ANDROID_NDK_HOME
set -gx ANDROID_PLATFORM 24
fish_add_path $ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin
cargo ndk -t arm64-v8a --platform 24 build --release

WINDOWS & LINUX
cargo build --release
*/

// #[cfg(not(test))]
#[allow(
    clippy::similar_names,
    reason = "that's what they're called in C, i don't give a fuck about clippy"
)]
#[cfg(any(target_os = "linux", target_os = "android"))]
#[unsafe(no_mangle)]
extern "C" fn main(argc: c_int, argv: *const *const c_char) -> c_int {
    let _ = ARGS.set(imp::env::args_init(argc as usize, argv.cast()));
    lazyfetch_main() as c_int
}

// #[cfg(not(test))]
#[cfg(target_os = "windows")]
#[unsafe(no_mangle)]
extern "C" fn main() -> c_int {
    let _ = ARGS.set(imp::env::args_init());
    lazyfetch_main() as c_int
}

fn lazyfetch_main() -> i32 {
    let args = env::args();

    // Commands
    if let Some(pos) = args.iter().position(|a| a == "--help" || a == "-h") {
        print_help(args.get(pos + 1).map(String::as_str))
    }

    if let Some(pos) = args.iter().position(|a| a == "--version" || a == "-v") {
        print_version(args.get(pos + 1).map(String::as_str))
    }

    // Config init
    let config = get_config(&mut args.iter());
    Config::get_or_init(config);

    // Logo init
    let name_raw = args.iter()
        .position(|a| a == "--logo" || a == "-l")
        .and_then(|pos| args.get(pos + 1).cloned())
        .or_else(|| Config::get().get_logo_name());

    let (logo_name, custom) =
        name_raw.map_or_else(|| {
            let id = crate::detect::os::get_id().as_boxed_str();
            (id, UILogo::Preset)
        }, |name| if name == "null" {
                let id = crate::detect::os::get_id().as_boxed_str();
                (id, UILogo::None)
            } else {
                get_logo_name_and_custom(&name)
            }
        );
    
    // Print logo and info
    match custom {
        UILogo::None => print_none(),
        UILogo::Preset => print_base(&logo_name),
        UILogo::Ascii(s) => print_ascii(s),
        UILogo::Image(i) => print_image(&i),
    }

    if args.iter().any(|a| a == "--alloc-report") {
        print_alloc_report();
    }

    if args.iter().any(|a| a == "--wait" || a == "-w") {
        loop {
            // SAFETY: Just a nop
            unsafe { core::arch::asm!("nop") };
        }
    }

    // The handle is created not with `GetStdHandle`,
    // but with `CreateFile`, which requires manual freeing
    #[cfg(target_os = "windows")]
    let _ = env::close_terminal_handle();
    NvidiaLib::drop_nvidia();

    0
}
