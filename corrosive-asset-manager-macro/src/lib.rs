use proc_macro::TokenStream;
use quote::ToTokens;
use syn::__private::quote::quote;
use syn::{parse_macro_input, parse_str, DeriveInput, LitStr};

#[proc_macro_derive(Asset)]
pub fn asset(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident.to_string();
    let static_name: proc_macro2::TokenStream =
        parse_str(format!("{}_ASSET", name.to_uppercase()).as_str()).unwrap();
    let asset_name: proc_macro2::TokenStream =
        LitStr::new(name.as_str(), input.ident.span()).to_token_stream();
    let name = input.ident.to_token_stream();
    (quote! {
            static #static_name: corrosive_asset_manager::asset_server::AssetServerObject<#name> = corrosive_asset_manager::asset_server::AssetServerObject {
            server: std::sync::LazyLock::new(|| std::sync::Mutex::new(Default::default())),
        };
        impl corrosive_asset_manager::asset_server::AssetObject for #name {
            fn get_server() -> &'static std::sync::Mutex<corrosive_asset_manager::asset_server::AssetServer<Self>>
            where
                Self: Sized,
            {
                &#static_name.server
            }
        }
    })
    .into()
}

#[proc_macro]
pub fn static_hasher(input: TokenStream) -> TokenStream {
    let input_literal = parse_macro_input!(input as LitStr);
    let input_str = input_literal.value();

    const FNV_OFFSET_BASIS: u64 = 14695981039346656037;
    const FNV_PRIME: u64 = 1099511628211;

    let mut hash = FNV_OFFSET_BASIS;
    for byte in input_str.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }

    let output = quote! {
        #hash
    };

    output.into()
}

fn main() {}
