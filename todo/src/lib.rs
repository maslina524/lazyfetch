#![allow(clippy::cargo_common_metadata)]
#![cfg_attr(unstable, feature(proc_macro_diagnostic))]
#![cfg_attr(unstable, feature(proc_macro_value))]

#[cfg(unstable)]
use {
    proc_macro::{Diagnostic, Level, Span, TokenTree},
    std::sync::atomic::{AtomicU32, Ordering::Relaxed},
};

use proc_macro::TokenStream;

#[cfg(unstable)]
static COUNTER: AtomicU32 = AtomicU32::new(1);

#[cfg(unstable)]
fn get_message(attr: TokenStream) -> Option<String> {
    attr.into_iter().next().and_then(|t| match t {
        TokenTree::Literal(s) => {
            let string = s.str_value().ok()?;
            Some(format!("#{} TODO: {string}", COUNTER.load(Relaxed)))
        }
        _ => None,
    })
}

#[allow(clippy::missing_panics_doc)]
#[cfg(unstable)]
#[proc_macro_attribute]
pub fn todo(attr: TokenStream, item: TokenStream) -> TokenStream {
    let message = get_message(attr)
        .unwrap_or_else(|| format!("#{} TODO: Not implemented", COUNTER.load(Relaxed)));

    let span = Span::mixed_site();
    Diagnostic::spanned(span, Level::Warning, &message).emit();
    COUNTER.fetch_add(1, Relaxed);
    item
}

#[cfg(not(unstable))]
#[proc_macro_attribute]
pub fn todo(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
