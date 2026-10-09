use alloc::{borrow::Cow, collections::BTreeMap, string::String};
use doc::Docs;

use crate::{
    formats, impl_display_for_module,
    json::Value,
    modules::{self, FormatValue, Module, Title},
    sync::OnceLock,
};

static SEPARATOR: OnceLock<Separator> = OnceLock::new();

#[derive(Debug, Docs)]
pub struct Separator;

impl Module for Separator {
    fn new() -> Self {
        Self {}
    }

    fn get() -> &'static Self {
        SEPARATOR.get_or_init(|| Self::new())
    }

    fn key(&self) -> &'static str {
        ""
    }

    fn title(&self) -> &'static str {
        ""
    }

    fn string_name(&self) -> &'static str {
        "separator"
    }

    fn format(
        &self,
        _key: super::FormatValue,
        _format: super::FormatValue,
        _map: Option<&BTreeMap<String, Value>>,
    ) -> Option<Cow<'_, str>> {
        let string = Title::get()
            .format(FormatValue::default(), FormatValue::default(), None)
            .map(|title| "-".repeat(formats::visible_len(&title)));

        string.map(modules::expand_env_in_module)
    }

    fn resolve_field(&self, _name: &str) -> Option<&dyn core::fmt::Display> {
        unreachable!()
    }
}

impl_display_for_module!(Separator);
