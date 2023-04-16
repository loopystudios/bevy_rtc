extern crate proc_macro;

use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

use proc_macro::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Payload)]
pub fn derive_payload_fn(item: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(item);
    let name = ident.to_token_stream();
    let mut s = DefaultHasher::new();
    name.to_string().hash(&mut s);
    let id = s.finish() as u16;
    quote! {
        impl ::silk_net::Message for #name {
            fn id() -> u16 {
                #id
            }
        }
    }
    .into()
}
