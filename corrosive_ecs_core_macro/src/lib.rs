use crate::task::task_macro::task_fn;
use proc_macro::TokenStream;
use quote::ToTokens;
use std::sync::{mpsc, Arc, LazyLock, Mutex, RwLock};
use syn::__private::quote::quote;
use syn::parse::Parse;
use syn::{parse_macro_input, Ident};

mod app;
mod component;
mod task;

#[doc = include_str!("../docs/task.md")]
#[proc_macro_attribute]
pub fn task(attr: TokenStream, item: TokenStream) -> TokenStream {
    task_fn(attr, item)
}

#[proc_macro]
pub fn remove_entity(_item: TokenStream) -> TokenStream {
    let ident = parse_macro_input!(_item as Ident);

    let output = quote! {
        fn print_ident() {
            println!("The identifier is: {}", stringify!(#ident));
        }
    };

    println!("\"{output}\"");

    TokenStream::from(output)
}

#[proc_macro]
pub fn corrosive_engine_builder(item: TokenStream) -> TokenStream {
    app::corrosive_engine_builder(item)
}

#[proc_macro_derive(Component)]
pub fn component(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}

#[proc_macro_derive(State)]
pub fn state(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}

#[proc_macro_derive(Resource)]
pub fn resource(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}
