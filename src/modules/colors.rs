use alloc::{
    string::String,
    collections::BTreeMap,
    borrow::Cow
};
use doc::Docs;

use crate::{
    format,
    impl_display_for_module, 
    modules::{self, Module},
    sync::OnceLock,
    json::Value
};

static COLORS        : OnceLock<Colors> = OnceLock::new();
static RANGE_BLOCK   : &[usize]         = &[30, 90];
static RANGE_NO_BLOCK: &[usize]         = &[30];

#[derive(Debug, Docs)]
pub struct Colors;

impl Module for Colors {
    fn new() -> Self {
        Self {}
    }

    fn get() -> &'static Self {
        COLORS.get_or_init(|| {
            Self::new()
        })
    }

    fn key(&self) -> &'static str {
        ""
    }

    fn title(&self) -> &'static str {
        ""
    }

    fn string_name(&self) -> &'static str {
        "colors"
    }

    fn format(&self, _key: super::FormatValue, _format: super::FormatValue, map: Option<&BTreeMap<String, Value>>) -> Option<Cow<'_, str>> {
        let binding = BTreeMap::new();
        let map = map.unwrap_or(&binding);

        let padding_left_num = map
            .get("paddingLeft")
            .unwrap_or(&Value::Null)
            .as_number()
            .unwrap_or(0.0) as usize;
        let padding_left = " ".repeat(padding_left_num);

        let symbol_map = map
            .get("symbol")
            .unwrap_or(&Value::Null)
            .as_string()
            .map_or_else(|| "block", String::as_str);

        let symbol = match symbol_map {
            "block" => "███",
            "circle" => "● ",
            _ => symbol_map
        };

        let ranges = if symbol_map == "block" {
            RANGE_BLOCK
        } else {
            RANGE_NO_BLOCK
        };

        let mut ret = String::with_capacity(8 * (symbol.len() + 5) * ranges.len());
        
        for r in ranges {
            let r = *r;
            ret.push_str(&padding_left);
            for i in r..=r + 7 {
                ret.push_str(&format!("\x1b[{i}m{symbol}"));
            }
            ret.push_str("\x1b[0m\n");
        }
        
        Some(modules::expand_env_in_module(ret))
    }

    fn resolve_field(&self, _name: &str) -> Option<&dyn core::fmt::Display> {
        unreachable!()
    }
}

impl_display_for_module!(Colors);