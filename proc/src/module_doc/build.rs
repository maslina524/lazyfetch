use proc_macro2::TokenStream;
use quote::quote;
use syn::Field;

use crate::module_doc::parser;

pub fn format(fields: &[&Field]) -> TokenStream {
    if fields.is_empty() {
        return quote! {
            fn strings_format() -> Option<&'static [crate::modules::DocString]> {
                None
            }
        };
    }

    let mut idx = 1;
    let strings = fields.iter().map(|field| {
        let name = parser::field_to_config_name(field);
        let typ = format!("{{{idx}}}");
        let desc =
            parser::get_doc_comment(field).map_or_else(|| quote! { None }, |s| quote! { Some(#s) });
        idx += 1;

        quote! {
            crate::modules::DocString { name: #name, second: #typ, desc: #desc }
        }
    });

    quote! {
        fn strings_format() -> Option<&'static [crate::modules::DocString]> {
            Some(&[
                #(#strings),*
            ])
        }
    }
}

pub fn lua(fields: &[&Field]) -> TokenStream {
    if fields.is_empty() {
        return quote! {
            fn strings_lua() -> Option<&'static [crate::modules::DocString]> {
                None
            }
        };
    }

    let strings = fields.iter().map(|field| {
        let name = parser::field_to_config_name(field);
        let field_ty = &field.ty;
        let desc = parser::get_doc_comment(field)
            .map_or_else(|| quote! { None }, |s| quote! { Some(#s) });

        quote! {
            crate::modules::DocString { name: #name, second: <#field_ty as crate::lua::AsLua>::LUA_TYPE, desc: #desc }
        }
    });

    quote! {
        fn strings_lua() -> Option<&'static [crate::modules::DocString]> {
            Some(&[
                #(#strings),*
            ])
        }
    }
}

pub fn example(fields: &[&Field]) -> TokenStream {
    if fields.is_empty() {
        return quote! {
            fn strings_example(self) -> Option<alloc::vec::Vec<(&'static str, alloc::string::String)>> {
                None
            }
        };
    }

    let strings = fields.iter().map(|field| {
        let name = parser::field_to_config_name(field);
        let ident = field.ident.as_ref().unwrap();

        quote! {
            (#name, crate::format!("{}", self.#ident))
        }
    });

    quote! {
        fn strings_example(self) -> Option<alloc::vec::Vec<(&'static str, alloc::string::String)>> {
            Some(alloc::vec![
                #(#strings),*
            ])
        }
    }
}
