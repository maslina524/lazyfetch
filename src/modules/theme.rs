use doc::Docs;

use crate::{
    detect::theme, 
    impl_display_for_module, 
    impl_module, 
    modules::Module, 
    str::SmolStr, 
    sync::OnceLock
};

static THEME: OnceLock<Theme> = OnceLock::new();

#[derive(Debug, Default, Docs)]
pub struct Theme {
    #[doc = "Theme part 1"]
    pub theme1: SmolStr,
    #[doc = "Theme part 2"]
    pub theme2: SmolStr
}

impl Module for Theme {
    fn new() -> Self {
        theme::get()
    }

    fn get() -> &'static Self {
        THEME.get_or_init(|| {
            Self::new()
        })
    }

    fn key(&self) -> &'static str {
        "Theme"
    }

    fn title(&self) -> &'static str {
        "{theme1}{theme2}"
    }

    fn string_name(&self) -> &'static str {
        "theme"
    }

    impl_module!(
        theme1, theme2
    );
}

impl_display_for_module!(Theme);