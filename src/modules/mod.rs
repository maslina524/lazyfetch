#![doc = include_str!("README.md")]

pub mod break_;     // 7)  Break         : Print an empty line
pub mod colors;     // 14) Colors        : Display the terminal's 16-color palette
pub mod commit;     // LF) Commit        : Display last commit
pub mod cpu;        // 15) CPU           : Print CPU name, frequency, etc.
pub mod custom;     // 19) Custom        : Print a custom string, with or without key
pub mod datetime;   // 20) DateTime      : Print the current date and time
pub mod disk;       // 22) Disk          : Print partitions, space usage, file system, etc
pub mod gpu;        // 29) GPU           : Print GPU names, memory sizes, types, etc
pub mod initsystem; // 32) InitSystem    : Print init system (pid 1) name and version
pub mod kernel;     // 33) Kernel        : Print system kernel version
pub mod locale;     // 37) Locale        : Print system locale name
pub mod memory;     // 41) Memory        : Print system memory usage information
pub mod os;         // 47) OS            : Print the OS or Linux distribution name and version
pub mod publicip;   // 52) PublicIp      : Print your public IP address, etc
pub mod processes;  // 53) Processes     : Print number of running processes
pub mod separator;  // 55) Separator     : Print a separator line
// pub mod swap;    // 58) Swap          : Print swap (paging file) space usage
pub mod title;      // 63) Title         : Print the title, including your username and hostname
pub mod theme;      // 65) Theme         : Print the current desktop environment theme
pub mod uptime;     // 66) Uptime        : Print how long the system has been running
pub mod version;    // 68) Version       : Print the Fastfetch version and build information
pub mod wallpaper;  // 70) Wallpaper     : Print the file path of the current wallpaper
pub mod weather;    // 71) Weather       : Print weather information

pub use break_::Break;
pub use colors::Colors;
pub use commit::Commit;
pub use cpu::Cpu;
pub use custom::Custom;
pub use datetime::Datetime;
pub use disk::{DiskList, Disk};
pub use gpu::Gpu;
pub use initsystem::Initsystem;
pub use kernel::Kernel;
pub use locale::Locale;
pub use memory::Memory;
pub use os::Os;
pub use publicip::PublicIP;
pub use processes::Processes;
pub use separator::Separator;
pub use title::Title;
pub use theme::Theme;
pub use uptime::Uptime;
pub use version::Version;
pub use wallpaper::Wallpaper;
pub use weather::Weather;

use alloc::{
    string::String,
    collections::BTreeMap,
    vec::Vec
};

use crate::json::Value;

type ModulePtr = &'static dyn Module;
type Registry  = (&'static str, fn() -> ModulePtr);
type Example   = (&'static str, String);

static UNSUPPORTED_FIELDS: [&str; 1] = ["{cmake-built-type}"];

static REGISTRY: &[Registry] = &[
    ("break",      || Break::get()),
    ("colors",     || Colors::get()),
    ("commit",     || Commit::get()),
    ("cpu",        || Cpu::get()),
    ("custom",     || Custom::get()),
    ("datetime",   || Datetime::get()),
    ("disk",       || DiskList::get()),
    ("gpu",        || Gpu::get()),
    ("initsystem", || Initsystem::get()),
    ("kernel",     || Kernel::get()),
    ("locale",     || Locale::get()),
    ("memory",     || Memory::get()),
    ("os",         || Os::get()),
    ("publicip",   || PublicIP::get()),
    ("processes",  || Processes::get()),
    ("separator",  || Separator::get()),
    ("title",      || Title::get()),
    ("theme",      || Theme::get()),
    ("uptime",     || Uptime::get()),
    ("version",    || Version::get()),
    ("wallpaper",  || Wallpaper::get()),
    ("weather",    || Weather::get()),
];

#[derive(Default, Clone, Copy)]
pub struct FormatValue<'a> {
    pub format: Option<&'a str>,
    pub color: Option<&'a str>
}

#[derive(Debug)]
pub struct DocString {
    pub name: &'static str,
    pub second: &'static str,
    pub desc: Option<&'static str>
}

// 473kb -> 428kb
pub trait Docs {
    fn strings_format() -> Option<&'static [DocString]>;
    fn strings_lua() -> Option<&'static [DocString]>;
    fn strings_example(self) -> Option<Vec<(&'static str, String)>>;
}

pub struct DocsVtable {
    pub format: fn() -> Option<&'static [DocString]>,
    pub lua: fn() -> Option<&'static [DocString]>,
    pub example: fn() -> Option<alloc::vec::Vec<Example>>
}

impl DocsVtable {
    pub fn from_str(name: &str) -> Option<Self> {
        match name {
            "break"      => Some(Self { format: Break::strings_format,      lua: Break::strings_lua,      example: || Break::strings_example(Break::new())                 }),
            "colors"     => Some(Self { format: Colors::strings_format,     lua: Colors::strings_lua,     example: || Colors::strings_example(Colors::new())               }),
            "commit"     => Some(Self { format: Commit::strings_format,     lua: Commit::strings_lua,     example: || Commit::strings_example(Commit::new())               }),
            "cpu"        => Some(Self { format: Cpu::strings_format,        lua: Cpu::strings_lua,        example: || Cpu::strings_example(Cpu::new())                     }),
            "custom"     => Some(Self { format: Custom::strings_format,     lua: Custom::strings_lua,     example: || Custom::strings_example(Custom::new())               }),
            "datetime"   => Some(Self { format: Datetime::strings_format,   lua: Datetime::strings_lua,   example: || Datetime::strings_example(Datetime::new())           }),
            "disk"       => Some(Self { format: Disk::strings_format,       lua: Disk::strings_lua,       example: || Disk::strings_example(DiskList::new().first_owned()) }),
            "gpu"        => Some(Self { format: Gpu::strings_format,        lua: Gpu::strings_lua,        example: || Gpu::strings_example(Gpu::new())                     }),
            "initsystem" => Some(Self { format: Initsystem::strings_format, lua: Initsystem::strings_lua, example: || Initsystem::strings_example(Initsystem::new())       }),
            "kernel"     => Some(Self { format: Kernel::strings_format,     lua: Kernel::strings_lua,     example: || Kernel::strings_example(Kernel::new())               }),
            "locale"     => Some(Self { format: Locale::strings_format,     lua: Locale::strings_lua,     example: || Locale::strings_example(Locale::new())               }),
            "memory"     => Some(Self { format: Memory::strings_format,     lua: Memory::strings_lua,     example: || Memory::strings_example(Memory::new())               }),
            "os"         => Some(Self { format: Os::strings_format,         lua: Os::strings_lua,         example: || Os::strings_example(Os::new())                       }),
            "publicip"   => Some(Self { format: PublicIP::strings_format,   lua: PublicIP::strings_lua,   example: || PublicIP::strings_example(PublicIP::new())           }),
            "processes"  => Some(Self { format: Processes::strings_format,  lua: Processes::strings_lua,  example: || Processes::strings_example(Processes::new())         }),
            "separator"  => Some(Self { format: Separator::strings_format,  lua: Separator::strings_lua,  example: || Separator::strings_example(Separator::new())         }),
            "title"      => Some(Self { format: Title::strings_format,      lua: Title::strings_lua,      example: || Title::strings_example(Title::new())                 }),
            "theme"      => Some(Self { format: Theme::strings_format,      lua: Theme::strings_lua,      example: || Theme::strings_example(Theme::new())                 }),
            "uptime"     => Some(Self { format: Uptime::strings_format,     lua: Uptime::strings_lua,     example: || Uptime::strings_example(Uptime::new())               }),
            "version"    => Some(Self { format: Version::strings_format,    lua: Version::strings_lua,    example: || Version::strings_example(Version::new())             }),
            "wallpaper"  => Some(Self { format: Wallpaper::strings_format,  lua: Wallpaper::strings_lua,  example: || Wallpaper::strings_example(Wallpaper::new())         }),
            "weather"    => Some(Self { format: Weather::strings_format,    lua: Weather::strings_lua,    example: || Weather::strings_example(Weather::new())             }),
            _ => None,
        }
    }
}

pub trait Module {
    fn new() -> Self where Self: Sized;
    fn get() -> &'static Self where Self: Sized;
    fn key(&self) -> &'static str;
    fn title(&self) -> &'static str;
    fn string_name(&self) -> &'static str;
    fn format(&self, key: FormatValue, format: FormatValue, map: Option<&BTreeMap<String, Value>>) -> Option<String>;
    fn resolve_field(&self, name: &str) -> Option<&dyn core::fmt::Display>;
}

pub fn from_preset_module(s: &str) -> Option<&'static dyn Module> {
    REGISTRY
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(s))
        .map(|(_, f)| f())
}

pub fn __field_name(raw: &'static str) -> &'static str {
    let trimmed = raw.strip_prefix("r#").unwrap_or(raw);
    if trimmed.contains('_') {
        alloc::boxed::Box::leak(trimmed.replace('_', "-").into_boxed_str())
    } else {
        trimmed
    }
}

// Name is the string we get from the config
// Field is the raw field name from the structure
//
// We need to compare these two strings 
// ignoring `r#` at the beginning of field and interpreting `_` as `-`
pub fn __eq_name_and_field(name: &str, field: &str) -> bool {
    field
        .strip_prefix("r#")
        .unwrap_or(field)
        .chars()
        .map(|c| if c == '_' { '-' } else { c })
        .eq(name.chars())
}

#[macro_export]
macro_rules! impl_display_for_module {
    ($name:ident) => {
        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self.format(
                    $crate::modules::FormatValue::default(),
                    $crate::modules::FormatValue::default(),
                    None,
                ) {
                    Some(s) => f.write_str(&s),
                    None => Ok(()),
                }
            }
        }
    };
    ($name:ident < $lt:lifetime >) => {
        impl core::fmt::Display for $name<$lt> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self.format(
                    $crate::modules::FormatValue::default(),
                    $crate::modules::FormatValue::default(),
                    None,
                ) {
                    Some(s) => f.write_str(&s),
                    None => Ok(()),
                }
            }
        }
    };
}

#[macro_export]
macro_rules! impl_module {
    ($($field:ident),*) => {
        #[allow(unused_variables)]
        fn resolve_field(&self, name: &str) -> Option<&dyn core::fmt::Display> {
            $(
                {
                    let field_name = stringify!($field);
                    if $crate::modules::__eq_name_and_field(name, field_name) {
                        return Some(&self.$field as &dyn core::fmt::Display);
                    }
                }
            )*
            None
        }

        fn format(
            &self, 
            key: super::FormatValue, 
            format: super::FormatValue, 
            _map: Option<&alloc::collections::BTreeMap<alloc::string::String, super::Value>>
        ) -> Option<alloc::string::String> {
            use core::fmt::Write;

            use alloc::string::String;

            let mut ret = String::with_capacity(128);

            let title_raw = format.format.unwrap_or(self.title());
            // Proccess Lua
            let body: String = if let Some(code) = title_raw.strip_prefix("lua:") {
                let code = $crate::lua::open_lua_file(code).into_owned();

                #[allow(unused_mut)]
                let mut vars = alloc::collections::BTreeMap::new();

                $(
                    let key_str = stringify!($field).trim_start_matches("r#");
                    vars.insert(
                        alloc::borrow::ToOwned::to_owned(key_str),
                        $crate::lua::AsLua::as_lua(&self.$field),
                    );
                )*

                $crate::lua::LuaLib::get().exec(&code, vars)
            } else {
                alloc::borrow::ToOwned::to_owned(title_raw)
            };

            // Substituting values into the module body
            let mut body_ret = String::with_capacity(96);
            let body_parser = $crate::field::parser::FormatParserIter::new(&body);
            for part in body_parser {
                match part {
                    $crate::field::parser::Part::Text(s) => {
                        let _ = write!(body_ret, "{s}");
                    },
                    $crate::field::parser::Part::Var(v) => {
                        if let Some(display) = self.resolve_field(v) {
                            let _ = write!(body_ret, "{}", display);
                        } else {
                            let _ = write!(body_ret, "{{{v}}}");
                        }
                    }
                }
            }

            if body_ret.is_empty() {
                return None
            }

            // Substituting values into the module key
            let mut key_ret = String::with_capacity(16);
            let key_parser = $crate::field::parser::FormatParserIter::new(key.format.unwrap_or(self.key()));
            for part in key_parser {
                match part {
                    $crate::field::parser::Part::Text(s) => {
                        let _ = write!(key_ret, "{s}");
                    },
                    $crate::field::parser::Part::Var(v) => {
                        if let Some(display) = self.resolve_field(v) {
                            let _ = write!(key_ret, "{}", display);
                        } else {
                            let _ = write!(key_ret, "{{{v}}}");
                        }
                    }
                }
            }

            if key_ret.is_empty() {
                return Some(body_ret);
            }

            let key_color = key.color.unwrap_or($crate::logo::LogoInfo::get().unwrap().color_keys);
            let separator = $crate::config::Config::get().get_display_separator();
            
            let _ = write!(ret, "\x1b[{key_color};1m{key_ret}\x1b[0m{separator}{body_ret}");
            Some(ret)
        }
    };
}