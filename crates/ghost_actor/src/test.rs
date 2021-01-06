use super::*;

#[test]
fn full_derive_test() {
    let result = try_fmt(ghost_actor_macro(quote::quote! {
        /// My ghost actor trait
        pub trait Api {
            fn test1(&mut self, input: String) -> BoxFuture<'static, Result<String, ()>>;
            fn test2(&mut self, input1: String, input2: u32) -> BoxFuture<'static, Result<u32, ()>>;
        }
    }));
    println!("GOT: {}", result);
}
