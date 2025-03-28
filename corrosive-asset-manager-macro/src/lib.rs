use proc_macro::TokenStream;
use quote::ToTokens;
use syn::__private::quote::quote;
use syn::{parse_macro_input, parse_str, DeriveInput, Ident, LitStr};

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
        static mut #static_name: std::cell::LazyCell<
            corrosive_asset_manager::AssetManagerObject<#name>,
        > = std::cell::LazyCell::new(|| corrosive_asset_manager::AssetManagerObject::new());

        impl corrosive_asset_manager::AssetObject for #name {
            type AssetType = #name;

            unsafe fn remove_asset(id: &u64) {
                #static_name
                    .ref_counts
                    .write()
                    .expect(format!("Could not remove {} from {}", id,#asset_name).as_str())
                    .remove(id);
                #static_name
                    .values
                    .write()
                    .expect(format!("Could not remove {} from {}", id,#asset_name).as_str())
                    .remove(id);
            }

            unsafe fn replace_asset(id: &u64, asset_object: Self::AssetType) {
                #static_name
                    .values
                    .write()
                    .expect(format!("Could not join {} from {}", id,#asset_name).as_str())
                    .get_mut(id)
                    .replace(&mut corrosive_asset_manager::AssetValue::Ready(
                        asset_object,
                    ));
            }

            unsafe fn add_asset<'a>(
                id: u64,
                asset_object: Self::AssetType,
            ) -> (
                &'a corrosive_asset_manager::AssetValue<'a, Self::AssetType>,
                &'a std::sync::atomic::AtomicUsize,
            ) {
                let ref_count: &std::sync::atomic::AtomicUsize = {
                    let mut ref_count = #static_name
                        .ref_counts
                        .write()
                        .expect(format!("Could not add {} to {}", id,#asset_name).as_str());
                    match ref_count.get_mut(&id) {
                        None => std::mem::transmute(
                            ref_count
                                .entry(id)
                                .or_insert(std::sync::atomic::AtomicUsize::new(0)),
                        ),
                        Some(t) => {
                            t.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                            std::mem::transmute(t)
                        }
                    }
                };
                let asset = std::mem::transmute(
                    #static_name
                        .values
                        .write()
                        .expect(format!("Could not add {} to {}", id,#asset_name).as_str())
                        .entry(id)
                        .or_insert(corrosive_asset_manager::AssetValue::Ready(asset_object)),
                );
                (asset, ref_count)
            }
            unsafe fn load_asset<'a>(
                id: u64,
                asset_object: impl FnOnce() -> Self::AssetType + Send + 'static,
            ) -> (
                &'a corrosive_asset_manager::AssetValue<'a, Self::AssetType>,
                &'a std::sync::atomic::AtomicUsize,
            ) {
                let ref_count: &std::sync::atomic::AtomicUsize = {
                    let mut ref_count = #static_name
                        .ref_counts
                        .write()
                        .expect(format!("Could not add {} to {}", id,#asset_name).as_str());
                    match ref_count.get_mut(&id) {
                        None => std::mem::transmute(
                            ref_count
                                .entry(id)
                                .or_insert(std::sync::atomic::AtomicUsize::new(0)),
                        ),
                        Some(t) => {
                            t.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                            std::mem::transmute(t)
                        }
                    }
                };
                let binding = std::mem::transmute(&#static_name.default_value.get().read());
                let asset: &corrosive_asset_manager::AssetValue<#name> = std::mem::transmute(
                    #static_name
                        .values
                        .write()
                        .expect(format!("Could not add {} to {}", id,#asset_name).as_str())
                        .entry(id)
                        .or_insert(corrosive_asset_manager::AssetValue::NotReady(binding)),
                );
                std::thread::spawn(move || {
                    #static_name
                        .values
                        .write()
                        .expect(format!("Could not add {} to {}", id,#asset_name).as_str())
                        .insert(
                            id,
                            corrosive_asset_manager::AssetValue::Ready(asset_object()),
                        )
                });
                (asset, ref_count)
            }

            unsafe fn set_default<'a>(asset_object: Self::AssetType)
            where
                <Self as corrosive_asset_manager::AssetObject>::AssetType:
                    corrosive_asset_manager::AssetObject,
            {
                #static_name
                    .default_value
                    .get()
                    .write(Some(asset_object));
            }
        }
    })
    .into()
}

fn main() {}
