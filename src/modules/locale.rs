use doc::Docs;

use crate::{
    detect::locale, impl_display_for_module, impl_module, modules::Module, str::SmolStr,
    sync::OnceLock,
};

static LOCALE: OnceLock<Locale> = OnceLock::new();

#[derive(Debug, Docs)]
pub struct Locale {
    #[doc = "Locale code"]
    pub result: SmolStr,
}

impl Module for Locale {
    fn new() -> Self {
        Self {
            result: locale::get(),
        }
    }

    fn get() -> &'static Self {
        LOCALE.get_or_init(|| Self::new())
    }

    fn key(&self) -> &'static str {
        "Locale"
    }

    fn title(&self) -> &'static str {
        "{result}"
    }

    fn string_name(&self) -> &'static str {
        "locale"
    }

    impl_module!(result);
}

impl_display_for_module!(Locale);
