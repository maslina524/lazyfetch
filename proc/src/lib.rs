#![allow(clippy::cargo_common_metadata)]
#![cfg_attr(unstable, feature(proc_macro_diagnostic))]
#![cfg_attr(unstable, feature(proc_macro_value))]

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

mod derive;
mod parser;
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
    derive::doc::imp(input)
}

// DOC MACRO
#[allow(clippy::missing_panics_doc)]
#[proc_macro_derive(Module)]
pub fn module_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let fields = parser::get_fields(&input.data);
    derive::module::imp(&input.ident, &fields).into()
}
