#![forbid(unsafe_code)]
#![forbid(warnings)]
#![forbid(missing_docs)]
//! A simple, ergonomic, idiomatic, macro for generating the boilerplate
//! to use rust futures tasks in a concurrent actor style.

use proc_macro::TokenStream as TokenStream1;

/// Provides the #[derive(GhostActor)] GhostActor derive macro.
#[proc_macro_derive(GhostActor)]
pub fn ghost_actor_derive1(input: TokenStream1) -> TokenStream1 {
    ghost_actor::ghost_actor_derive(input.into()).into()
}
