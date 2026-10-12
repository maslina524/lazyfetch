use proc_macro2::TokenStream;
use quote::quote;
use syn::{Field, Ident, LitStr};

#[inline]
fn config_key(raw: &str) -> String {
    raw.strip_prefix("r#").unwrap_or(raw).replace('_', "-")
}

pub fn imp(type_name: &Ident, fields: &[&Field]) -> TokenStream {
    let resolve_arms = fields.iter().map(|field| {
        let ident = field
            .ident
            .as_ref()
            .expect("impl_module: field must be named");

        let normalized = config_key(&ident.to_string());
        let name_lit = LitStr::new(&normalized, ident.span());

        quote! {
            #name_lit => ::core::option::Option::Some(
                &this.#ident as &dyn ::core::fmt::Display
            ),
        }
    });

    let lua_var_inserts = fields.iter().map(|field| {
        let ident = field
            .ident
            .as_ref()
            .expect("impl_module: field must be named");

        let raw = ident.to_string();
        let key = raw.strip_prefix("r#").unwrap_or(&raw);
        let name_lit = LitStr::new(key, ident.span());

        quote! {
            vars.insert(
                ::alloc::string::String::from(#name_lit),
                crate::lua::AsLua::as_lua(&self.#ident),
            );
        }
    });

    quote! {
        impl crate::modules::ModuleFormat for #type_name {
            #[inline]
            fn format(
                &self,
                key: crate::modules::FormatValue<'_>,
                title: crate::modules::FormatValue<'_>,
                _map: ::core::option::Option<
                    &::alloc::collections::BTreeMap<
                        ::alloc::string::String,
                        crate::json::Value,
                    >,
                >,
            ) -> ::core::option::Option<::alloc::borrow::Cow<'_, str>> {
                use ::alloc::string::String;
                use ::core::fmt::Write;

                #[inline]
                fn resolve_field<'a>(
                    this: &'a #type_name,
                    name: &str,
                ) -> Option<&'a dyn ::core::fmt::Display> {
                    match name {
                        #(#resolve_arms)*
                        _ => None,
                    }
                }

                let mut ret = String::with_capacity(128);

                let title_raw = title.format.unwrap_or(self.title());

                // Proccess Lua
                let body: String = if let Some(code) =
                    title_raw.strip_prefix("lua:")
                {
                    let code = crate::lua::open_lua_file(code).into_owned();

                    #[allow(unused_mut)]
                    let mut vars = ::alloc::collections::BTreeMap::new();

                    #(#lua_var_inserts)*

                    crate::lua::LuaLib::get().exec(&code, vars)
                } else {
                    ::alloc::borrow::ToOwned::to_owned(title_raw)
                };

                // Substituting values into the module body
                let mut body_ret = String::with_capacity(96);
                let body_parser = crate::field::parser::FormatParserIter::new(&body);
                for part in body_parser {
                    match part {
                        crate::field::parser::Part::Text(s) => {
                            let _ = write!(body_ret, "{s}");
                        },
                        crate::field::parser::Part::Var(v) => {
                            if let Some(display) =
                                resolve_field(self, v)
                            {
                                let _ = write!(body_ret, "{}", display);
                            } else {
                                let _ = write!(body_ret, "{{{v}}}");
                            }
                        }
                    }
                }

                if body_ret.is_empty() {
                    return None;
                }

                // Substituting values into the module key
                let mut key_ret = String::with_capacity(16);
                let key_parser =
                    crate::field::parser::FormatParserIter::new(key.format.unwrap_or(self.key()));
                for part in key_parser {
                    match part {
                        crate::field::parser::Part::Text(s) => {
                            let _ = write!(key_ret, "{s}");
                        },
                        crate::field::parser::Part::Var(v) => {
                            if let Some(display) = resolve_field(self, v) {
                                let _ = write!(key_ret, "{}", display);
                            } else {
                                let _ = write!(key_ret, "{{{v}}}");
                            }
                        }
                    }
                }

                if key_ret.is_empty() {
                    let expanded = crate::modules::expand_env_in_module(body_ret);
                    return Some(expanded);
                }

                let key_color = key
                    .color
                    .unwrap_or(crate::logo::LogoInfo::get().unwrap().color_keys);
                let separator = crate::config::Config::get().get_display_separator();

                let _ = write!(
                    ret,
                    "\x1b[{key_color};1m{key_ret}\x1b[0m{separator}{body_ret}"
                );
                let expanded = crate::modules::expand_env_in_module(ret);
                Some(expanded)
            }
        }
    }
}
