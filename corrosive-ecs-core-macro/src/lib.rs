use proc_macro::TokenStream;
use syn::__private::quote::quote;
use syn::{parse_macro_input, Ident};

mod app;
mod component;
mod task;

/// Used to mark functions as tasks.
#[proc_macro_attribute]
pub fn task(attr: TokenStream, item: TokenStream) -> TokenStream {
    //task_fn(attr, item)
    item
}
/// Used to schedule the engine.
#[proc_macro]
pub fn corrosive_engine_builder(item: TokenStream) -> TokenStream {
    app::corrosive_engine_builder(item).into()
}

/// Used to mark structs & enums as a component.
#[proc_macro_derive(Component)]
pub fn component(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}
/// Used to mark structs & enums as a state.
#[proc_macro_derive(State)]
pub fn state(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}
/// Used to mark structs & enums as a resource.
#[proc_macro_derive(Resource)]
pub fn resource(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}
/// Used to mark structs & enums as a TraitBound.
#[proc_macro_attribute]
pub fn trait_bound(attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
