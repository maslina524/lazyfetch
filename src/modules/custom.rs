use doc::Docs;

use crate::{impl_display_for_module, impl_module, modules::Module, sync::OnceLock};

static CUSTOM: OnceLock<Custom> = OnceLock::new();

#[derive(Debug, Docs)]
pub struct Custom;

impl Module for Custom {
    fn new() -> Self {
        Self {}
    }

    fn get() -> &'static Self {
        CUSTOM.get_or_init(Self::new)
    }

    fn key(&self) -> &'static str {
        "Custom"
    }

    fn title(&self) -> &'static str {
        ""
    }

    fn string_name(&self) -> &'static str {
        "custom"
    }

    impl_module!();
}

impl_display_for_module!(Custom);
