use crate::{
    modules::{Module, Os, theme::Theme}, 
    str::SmolStr
};

pub fn get() -> Theme {
    let ver_str = &Os::get().version;
    ver_str.parse::<u32>().map_or_default(|ver| {
        let theme1 = match ver {
            v if (10..).contains(&v) => SmolStr::from_static("Fluent"),
            v if (8..10).contains(&v) => SmolStr::from_static("Metro"),
            _ => SmolStr::from_static("Aero")
        };
        Theme { theme1, theme2: SmolStr::empty() }
    })
}