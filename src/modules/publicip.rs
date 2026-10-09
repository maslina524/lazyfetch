use doc::Docs;

use alloc::string::String;

use crate::{
    detect::publicip, impl_display_for_module, impl_module, modules::Module, sync::OnceLock,
};

static PUBLICIP: OnceLock<PublicIP> = OnceLock::new();

#[derive(Debug, Docs, Default)]
pub struct PublicIP {
    #[doc = "Public IP address"]
    pub ip: String,
    #[doc = "Location"]
    pub location: String,
}

impl Module for PublicIP {
    fn new() -> Self {
        publicip::get()
    }

    fn get() -> &'static Self {
        PUBLICIP.get_or_init(|| Self::new())
    }

    fn key(&self) -> &'static str {
        "PublicIP"
    }

    fn title(&self) -> &'static str {
        "{ip} ({location})"
    }

    fn string_name(&self) -> &'static str {
        "publicip"
    }

    impl_module!(ip, location);
}

impl_display_for_module!(PublicIP);
