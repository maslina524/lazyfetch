use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

pub mod build;
use crate::parser;

pub fn imp(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    let fields = parser::get_fields(&input.data);

    let format = build::format(&fields); // BUILD -h module-format
    let lua = build::lua(&fields); // BUILD -h module-lua
    let example = build::example(&fields); // BUILD -h module-example

    let combined = quote! {
        use crate::lua::AsLua;

        impl crate::modules::Docs for #struct_name {
            #format
            #lua
            #example
        }
    };

    TokenStream::from(combined)
}
