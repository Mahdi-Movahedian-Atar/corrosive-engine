pub mod task_macro {
    use corrosive_ecs_core::build::codegen::generate_task_body;
    use corrosive_ecs_core::build::tasks_scan::get_task_input;
    use proc_macro2::TokenStream;
    use quote::{quote, ToTokens};
    use syn::token::RArrow;
    use syn::{
        parse2, parse_macro_input, parse_quote, parse_str, Item, ItemFn, ReturnType, Stmt, Token,
        Type,
    };

    pub fn task_fn(
        attr: proc_macro::TokenStream,
        item: proc_macro::TokenStream,
    ) -> proc_macro::TokenStream {
        let c = item.clone();
        let mut body = parse_macro_input!(item as ItemFn);

        let old_inputs = get_task_input(body.sig.inputs.clone());
        let mut new_input: TokenStream = TokenStream::new();

        let mut index: usize = 0;
        for old_input in old_inputs.0 {
            let name: TokenStream = parse_str(old_input.0.as_str()).unwrap();
            let type_name: TokenStream =
                parse_str(format!("{}{}", body.sig.ident.to_string(), index).as_str()).unwrap();
            new_input.extend(quote! {#name: corrosive_engine::arch_types::arch_types::#type_name,});

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

        /*match &mut body.sig.output {
            ReturnType::Default => {
                // If the original return type is (), change it to String
                body.sig.output = ReturnType::Type((syn::parse_quote!(->).span(), Box::new(syn::parse_quote!(String))), Box::new(()));
            }
            ReturnType::Type(_, ty) => {
                *ty = syn::parse_quote!(Option<#ty>);
            }
        }

        let a: RArrow = parse_quote!(->);
        let v: Type = parse_quote!((#out));*/
        body.sig.output = ReturnType::Type(parse_quote!(->), Box::new(parse_quote!((#out))));

        //println!("{}", generate_task_body(&mut body.block.stmts,).to_string());
        //println!("{}", new_input.to_string());

        let a: TokenStream = quote! {#body}.into();
        //println!("{}", a.to_string());
        //a.into();
        c
    }
}
