#[cfg(not(unstable))]
mod stable {
    use proc_macro::TokenStream;
    pub fn imp(_attr: TokenStream) {}
}

#[cfg(unstable)]
mod unstable {
    use {
        proc_macro::{Diagnostic, Level, Span, TokenStream, TokenTree},
        std::sync::atomic::{AtomicU32, Ordering::Relaxed},
    };

    static COUNTER: AtomicU32 = AtomicU32::new(1);

    fn get_message(attr: TokenStream) -> Option<String> {
        attr.into_iter().next().and_then(|t| match t {
            TokenTree::Literal(s) => {
                let string = s.str_value().ok()?;
                Some(format!("#{} TODO: {string}", COUNTER.load(Relaxed)))
            }
            _ => None,
        })
    }

    pub fn imp(attr: TokenStream) {
        let message = get_message(attr)
            .unwrap_or_else(|| format!("#{} TODO: Not implemented", COUNTER.load(Relaxed)));

        let span = Span::mixed_site();
        Diagnostic::spanned(span, Level::Warning, &message).emit();
        COUNTER.fetch_add(1, Relaxed);
    }
}

#[cfg(not(unstable))]
pub use stable::imp;

#[cfg(unstable)]
pub use unstable::imp;
