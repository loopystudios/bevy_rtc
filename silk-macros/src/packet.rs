extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Packet)]
pub fn derive_packet_fn(item: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(item);
    let name = ident.to_token_stream();
    quote! {
        unsafe impl ::core::marker::Send for $name {}
        unsafe impl ::core::marker::Sync for $name {}
        #[derive(::core::fmt::Debug, ::std::default::Default, ::core::clone::Clone)]
        #[derive(::serde::Serialize)]
        $item
    }
    .into()
}
