pub mod task_macro {
    use proc_macro::TokenStream;
    use quote::quote;
    use syn::{parse_macro_input, ItemFn, Stmt};

    enum TaskType {
        Update,
        FixedUpdate,
        LongUpdate,
        SyncUpdate,
    }
    struct TaskInput {
        r#type: Option<TaskType>,
        position: Option<String>,
        place: Option<String>,
    }

    pub fn task_fn(attr: TokenStream, item: TokenStream) -> TokenStream {
        let body = parse_macro_input!(item as ItemFn);

        // Parse the macro arguments
        eprintln!("{}", &attr);
        eprintln!("+++++++++++++++++++++++++++++");
        // eprintln!("{}", item.clone());

        let mut found_macros = Vec::new();

        for stmt in &body.block.stmts {
            if let Stmt::Macro(mac) = stmt {
                if mac.mac.path.is_ident("add_entity") {
                    found_macros.push(mac.mac.clone());
                }
            }
        }

        // Print found macros
        for mac in &found_macros {
            eprintln!("Found macro: {}", quote! { #mac });
        }

        // Return the original function unchanged
        TokenStream::from(quote! { #body })
    }
}
