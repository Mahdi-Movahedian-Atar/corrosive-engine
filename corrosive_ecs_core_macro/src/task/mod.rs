pub mod task_macro {
    use corrosive_ecs_core::build::codegen::{generate_task_body, write_rust_file};
    use corrosive_ecs_core::build::tasks_scan::get_task_input;
    use proc_macro2::TokenStream;
    use quote::{quote, quote_spanned, ToTokens};
    use std::env;
    use syn::token::RArrow;
    use syn::{
        parse2, parse_macro_input, parse_quote, parse_str, Item, ItemFn, ReturnType, Stmt, Token,
        Type,
    };

    pub fn task_fn(
        attr: proc_macro::TokenStream,
        item: proc_macro::TokenStream,
    ) -> proc_macro::TokenStream {
        let original = item.clone();
        let mut body = parse_macro_input!(item as ItemFn);

        let old_inputs = get_task_input(body.sig.inputs.clone());
        let mut new_input: TokenStream = TokenStream::new();

        let mut index: usize = 0;
        for old_input in old_inputs.0 {
            let name: TokenStream = parse_str(old_input.0.as_str()).unwrap();
            let mut i_type: TokenStream = TokenStream::new();
            old_input.1.iter().for_each(|v| {
                i_type.extend(parse_str::<TokenStream>(format!("&{},", v).as_str()).unwrap());
            });
            new_input.extend(quote! {#name: corrosive_ecs_core::ecs_core::Arch<(#i_type)>,});

            index += 1;
        }
        for old_input in old_inputs.1 {
            let name: TokenStream = parse_str(old_input.0.as_str()).unwrap();
            let types: TokenStream = parse_str(old_input.1.as_str()).unwrap();

            new_input.extend(quote! {#name: corrosive_ecs_core::ecs_core::Res<#types>,});
        }
        for old_input in old_inputs.2 {
            let name: TokenStream = parse_str(old_input.0.as_str()).unwrap();
            let types: TokenStream = parse_str(old_input.1.as_str()).unwrap();

            new_input.extend(quote! {#name: corrosive_ecs_core::ecs_core::State<#types>,});
        }
        if let Some(t) = old_inputs.3 {
            let name: TokenStream = parse_str(t.as_str()).unwrap();
            new_input.extend(quote! {#name: corrosive_ecs_core::ecs_core::DeltaTime,});
        }

        body.sig.inputs = parse_quote! {#new_input};

        let out = generate_task_body(&mut body.block.stmts);
        body.sig.output = ReturnType::Type(parse_quote!(->), Box::new(parse_quote!((#out))));

        let new_body: TokenStream = quote! {#body}.into();

        new_body.into()
    }
}
