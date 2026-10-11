#![allow(clippy::cargo_common_metadata)]
#![cfg_attr(unstable, feature(proc_macro_diagnostic))]
#![cfg_attr(unstable, feature(proc_macro_value))]

use proc_macro::TokenStream;

mod module_doc;
mod todo;

// TODO MACRO
#[proc_macro_attribute]
pub fn todo(attr: TokenStream, item: TokenStream) -> TokenStream {
    todo::imp(attr.clone());
    item
}

// DOC MACRO
#[allow(clippy::missing_panics_doc)]
#[proc_macro_derive(Docs)]
pub fn docs_derive(input: TokenStream) -> TokenStream {
    module_doc::derive_imp(input)
}
