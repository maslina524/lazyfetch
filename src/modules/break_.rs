use alloc::{
    borrow::{Cow, ToOwned},
    collections::BTreeMap,
    string::String,
};

use doc::Docs;

use crate::{
    impl_display_for_module,
    json::Value,
    modules::{self, Module},
    sync::OnceLock,
};

static BREAK: OnceLock<Break> = OnceLock::new();

#[derive(Debug, Docs)]
pub struct Break;

impl Module for Break {
    fn new() -> Self {
        Self {}
    }

    fn get() -> &'static Self {
        BREAK.get_or_init(|| Self::new())
    }

    fn key(&self) -> &'static str {
        ""
    }

    fn title(&self) -> &'static str {
        "\n"
    }

    fn string_name(&self) -> &'static str {
        "break"
    }

    fn format(
        &self,
        _key: super::FormatValue,
        format: super::FormatValue,
        _map: Option<&BTreeMap<String, Value>>,
    ) -> Option<Cow<'_, str>> {
        let title_str = format.format.unwrap_or_else(|| self.title());
        Some(modules::expand_env_in_module(title_str.to_owned()))
    }

    fn resolve_field(&self, _name: &str) -> Option<&dyn core::fmt::Display> {
        unreachable!()
    }
}

impl_display_for_module!(Break);
