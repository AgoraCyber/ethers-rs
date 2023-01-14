use std::fmt::Display;

use proc_macro2::Span;
use quote::ToTokens;

pub trait ToSynError<T> {
    fn map_syn_error(self, span: Span) -> syn::Result<T>;

    fn map_syn_error_with<U: ToTokens>(self, tokens: U) -> syn::Result<T>;
}

impl<T, E> ToSynError<T> for Result<T, E>
where
    E: Display,
{
    fn map_syn_error(self, span: Span) -> syn::Result<T> {
        self.map_err(|err| syn::Error::new(span, format!("{}", err)))
    }

    fn map_syn_error_with<U: ToTokens>(self, tokens: U) -> syn::Result<T> {
        self.map_err(|err| syn::Error::new_spanned(tokens, format!("{}", err)))
    }
}
