extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Payload)]
pub fn derive_payload_fn(item: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(item);
    let mut s = DefaultHasher::new();
    ident.to_string().hash(&mut s);
    let id = s.finish() as u16;
    let reflect_name = ident.to_string();
    quote! {
        impl bevy_rtc::protocol::Payload for #ident {
            fn id() -> u16 {
                #id
            }
            fn reflect_name() -> &'static str {
                #reflect_name
            }
        }
    }
    .into()
}
