#![forbid(unsafe_code)]
#![forbid(warnings)]
#![forbid(missing_docs)]
//! A simple, ergonomic, idiomatic, macro for generating the boilerplate
//! to use rust futures tasks in a concurrent actor style.

use proc_macro::TokenStream as TokenStream1;

/// Provides the #[ghost_actor] trait attribute macro.
#[proc_macro_attribute]
pub fn ghost_actor(_attrs: TokenStream1, input: TokenStream1) -> TokenStream1 {
    ghost_actor::ghost_actor_macro(input.into()).into()
}
