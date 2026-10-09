#![doc = include_str!("README.md")]

pub mod break_; // 7)  Break         : Print an empty line
pub mod colors; // 14) Colors        : Display the terminal's 16-color palette
pub mod commit; // LF) Commit        : Display last commit
pub mod cpu; // 15) CPU           : Print CPU name, frequency, etc.
pub mod custom; // 19) Custom        : Print a custom string, with or without key
pub mod datetime; // 20) DateTime      : Print the current date and time
pub mod disk; // 22) Disk          : Print partitions, space usage, file system, etc
pub mod gpu; // 29) GPU           : Print GPU names, memory sizes, types, etc
pub mod initsystem; // 32) InitSystem    : Print init system (pid 1) name and version
pub mod kernel; // 33) Kernel        : Print system kernel version
pub mod locale; // 37) Locale        : Print system locale name
pub mod memory; // 41) Memory        : Print system memory usage information
pub mod os; // 47) OS            : Print the OS or Linux distribution name and version
pub mod processes; // 53) Processes     : Print number of running processes
pub mod publicip; // 52) PublicIp      : Print your public IP address, etc
pub mod separator; // 55) Separator     : Print a separator line
pub mod shell; // 56) Shell         : Print the current shell name and version
// pub mod swap; // 58) Swap          : Print swap (paging file) space usage
pub mod terminal; // 59) Terminal      : Print the current terminal name and version
pub mod theme; // 65) Theme         : Print the current desktop environment theme
pub mod title; // 63) Title         : Print the title, including your username and hostname
pub mod uptime; // 66) Uptime        : Print how long the system has been running
pub mod version; // 68) Version       : Print the Fastfetch version and build information
pub mod wallpaper; // 70) Wallpaper     : Print the file path of the current wallpaper
pub mod weather; // 71) Weather       : Print weather information

pub use break_::Break;
pub use colors::Colors;
pub use commit::Commit;
pub use cpu::Cpu;
pub use custom::Custom;
pub use datetime::Datetime;
pub use disk::{Disk, DiskList};
pub use gpu::Gpu;
pub use initsystem::Initsystem;
pub use kernel::Kernel;
pub use locale::Locale;
pub use memory::Memory;
pub use os::Os;
pub use processes::Processes;
pub use publicip::PublicIP;
pub use separator::Separator;
pub use shell::Shell;
pub use terminal::Terminal;
pub use theme::Theme;
pub use title::Title;
pub use uptime::Uptime;
pub use version::Version;
pub use wallpaper::Wallpaper;
pub use weather::Weather;

use alloc::{borrow::Cow, collections::BTreeMap, string::String, vec::Vec};

use crate::json::Value;

macro_rules! module_registry {
    (
        $(
            $name:literal => $registry_ty:ty
                $(, docs = $docs_ty:ty, example = $example:expr)?
            ;
        )*
    ) => {
        static REGISTRY: &[Registry] = &[
            $(($name, || <$registry_ty>::get()),)*
        ];

        impl DocsVtable {
            pub fn from_str(name: &str) -> Option<Self> {
                match name {
                    $(
                        $name => Some(module_registry!(
                            @entry $registry_ty $(, $docs_ty, $example)?
                        )),
                    )*
                    _ => None,
                }
            }
        }
    };
    (@entry $registry_ty:ty) => {
        Self {
            format:  <$registry_ty>::strings_format,
            lua:     <$registry_ty>::strings_lua,
            example: || <$registry_ty>::strings_example(<$registry_ty>::new()),
        }
    };
    (@entry $registry_ty:ty, $docs_ty:ty, $example:expr) => {
        Self {
            format:  <$docs_ty>::strings_format,
            lua:     <$docs_ty>::strings_lua,
            example: || <$docs_ty>::strings_example($example),
        }
    };
}

type ModulePtr = &'static dyn Module;
type Registry = (&'static str, fn() -> ModulePtr);
type Example = (&'static str, String);

static UNSUPPORTED_FIELDS: [&str; 1] = ["{cmake-built-type}"];

module_registry! {
    "break"      => Break;
    "colors"     => Colors;
    "commit"     => Commit;
    "cpu"        => Cpu;
    "custom"     => Custom;
    "datetime"   => Datetime;
    "disk"       => DiskList, docs = Disk, example = DiskList::new().first_owned();
    "gpu"        => Gpu;
    "initsystem" => Initsystem;
    "kernel"     => Kernel;
    "locale"     => Locale;
    "memory"     => Memory;
    "os"         => Os;
    "publicip"   => PublicIP;
    "processes"  => Processes;
    "separator"  => Separator;
    "shell"      => Shell;
    "terminal"   => Terminal;
    "title"      => Title;
    "theme"      => Theme;
    "uptime"     => Uptime;
    "version"    => Version;
    "wallpaper"  => Wallpaper;
    "weather"    => Weather;
}

#[derive(Default, Clone, Copy)]
pub struct FormatValue<'a> {
    pub format: Option<&'a str>,
    pub color: Option<&'a str>,
}

#[derive(Debug)]
pub struct DocString {
    pub name: &'static str,
    pub second: &'static str,
    pub desc: Option<&'static str>,
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
    pub example: fn() -> Option<alloc::vec::Vec<Example>>,
}

pub trait Module {
    fn new() -> Self
    where
        Self: Sized;
    fn get() -> &'static Self
    where
        Self: Sized;
    fn key(&self) -> &'static str;
    fn title(&self) -> &'static str;
    fn string_name(&self) -> &'static str;
    fn format(
        &self,
        key: FormatValue,
        title: FormatValue,
        map: Option<&BTreeMap<String, Value>>,
    ) -> Option<Cow<'_, str>>;
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

#[cfg(target_os = "windows")]
pub fn expand_env_in_module(s: String) -> Cow<'static, str> {
    match crate::windows::env::expand_env(&s) {
        Ok(alloc::borrow::Cow::Borrowed(_)) => s.into(),
        Ok(alloc::borrow::Cow::Owned(expanded)) => expanded.into(),
        Err(e) => {
            crate::warning!("Failed to convert Utf16: {e}");
            s.into()
        }
    }
}

#[cfg(target_family = "unix")]
pub const fn expand_env_in_module(s: String) -> Cow<'static, str> {
    Cow::Owned(s)
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
            title: super::FormatValue,
            _map: Option<&alloc::collections::BTreeMap<alloc::string::String, super::Value>>
        ) -> Option<alloc::borrow::Cow<'_, str>> {
            use core::fmt::Write;

            use alloc::string::String;

            let mut ret = String::with_capacity(128);

            let title_raw = title.format.unwrap_or(self.title());
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
                let expanded = $crate::modules::expand_env_in_module(body_ret);
                return Some(expanded);
            }

            let key_color = key.color.unwrap_or($crate::logo::LogoInfo::get().unwrap().color_keys);
            let separator = $crate::config::Config::get().get_display_separator();

            let _ = write!(ret, "\x1b[{key_color};1m{key_ret}\x1b[0m{separator}{body_ret}");
            let expanded = $crate::modules::expand_env_in_module(ret);
            Some(expanded)
        }
    };
}
