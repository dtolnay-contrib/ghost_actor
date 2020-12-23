use super::*;

#[test]
fn full_derive_test() {
    let result = try_fmt(ghost_actor_derive(quote::quote! {
        /// My ghost actor trait
        pub trait Api {
            fn test1(&mut self, input: String) -> BoxFuture<'static, Result<String, ()>>;
        }
    }));
    println!("GOT: {}", result);
}
