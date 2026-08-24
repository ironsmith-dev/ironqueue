//! Procedural macros for the `ironqueue` crate.
//!
//! Do not depend on this crate directly; use the re-export at `ironqueue::job`.

use proc_macro::TokenStream;

mod attrs;
mod expand;

/// Marks an `async fn` as an ironqueue job handler.
///
/// The accepted attributes and the signature contract are documented on the
/// re-export this is used through, `ironqueue::job`.
#[proc_macro_attribute]
pub fn job(attr: TokenStream, item: TokenStream) -> TokenStream {
    expand::expand_job(attr.into(), item.into()).unwrap_or_else(syn::Error::into_compile_error).into()
}

/// Marks an `async fn` as an ironqueue cron job, run on the given schedule.
///
/// The first argument is the cron expression; its syntax and whether it can
/// ever produce a UTC occurrence are checked at compile time. The rest are the
/// same configuration attributes as `job`. Cron functions take no payload —
/// every parameter is an extractor.
///
/// The accepted attributes are documented on the re-export this is used
/// through, `ironqueue::cron`.
#[proc_macro_attribute]
pub fn cron(attr: TokenStream, item: TokenStream) -> TokenStream {
    expand::expand_cron(attr.into(), item.into()).unwrap_or_else(syn::Error::into_compile_error).into()
}
