use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, parse_str, DeriveInput, LitStr};

#[proc_macro_derive(Material2D)]
pub fn material_2d(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident.to_string();
    let wrapper_name: proc_macro2::TokenStream =
        parse_str(format!("{}Wrapper", name).as_str()).unwrap();
    let name = input.ident.to_token_stream();
    (quote! {
            struct #wrapper_name {
                asset: Asset<#name>,
            }
            impl #wrapper_name for StandardMaterial2DWrapper {
                fn get_bind_group(&self) -> &'static BindGroup {
                    self.asset.get().get_bind_group()
                }
            }
            impl Material2D for #name {
                fn generate_wrapper(&self, asset: Asset<Self>) -> Box<dyn Material2DWrapper + Send + Sync> {
                    Box::new(#wrapper_name { asset })
                }
            }
    })
        .into()
}
