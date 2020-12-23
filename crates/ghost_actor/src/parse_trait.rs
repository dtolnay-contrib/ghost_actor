use crate::*;

#[derive(Debug)]
pub struct ParsedFn {
    pub name: syn::Ident,
    pub receiver: TokenStream,
    pub args: Vec<(TokenStream, TokenStream)>,
    pub output: TokenStream,
}

#[derive(Debug)]
pub struct ParsedTrait {
    pub name: syn::Ident,
    pub fns: Vec<ParsedFn>,
}

pub fn parse_trait(input: TokenStream) -> ParsedTrait {
    let p_trait: syn::ItemTrait = syn::parse2(input).unwrap();

    let name = p_trait.ident.clone();

    let mut fns = Vec::new();
    for item in p_trait.items {
        if let syn::TraitItem::Method(method) = item {
            let name = method.sig.ident.clone();

            let mut receiver = None;
            let mut args = Vec::new();
            for arg in method.sig.inputs.iter() {
                match arg {
                    syn::FnArg::Receiver(r) => {
                        receiver = Some(r.to_token_stream());
                    }
                    syn::FnArg::Typed(t) => {
                        let name = t.pat.to_token_stream();
                        let ty = t.ty.to_token_stream();
                        args.push((name, ty));
                    }
                }
            }

            let output = match method.sig.output {
                syn::ReturnType::Type(_, ty) => ty.to_token_stream(),
                _ => {
                    panic!("ghost actor trait fns must return GhostActorResult")
                }
            };

            let receiver =
                receiver.expect("ghost actor trait fns must take &mut self");

            fns.push(ParsedFn {
                name,
                receiver,
                args,
                output,
            });
        }
    }

    let out = ParsedTrait { name, fns };

    println!("{:#?}", out);
    out
}
