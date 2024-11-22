use proc_macro::TokenStream;
use quote::ToTokens;
use syn::__private::quote::quote;
use syn::{parse_macro_input, Expr, Ident, LitStr, Token};
use syn::parse::{Parse, ParseStream};

struct MacroArgs {
    string_arg: Option<LitStr>,
    enum_arg: Ident,
    third_arg: Option<Expr>,
}

impl Parse for MacroArgs {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let string_arg = if input.peek(LitStr) {
            Some(input.parse::<LitStr>()?)
        } else {
            None
        };

        // Parse comma after the first argument, if present
        if string_arg.is_some() && input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }

        // Parse the second argument (enum - required)
        let enum_arg = input.parse::<Ident>()?;

        // Parse comma after the second argument, if present
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }

        // Parse the third argument (optional function or string)
        let third_arg = if !input.is_empty() {
            Some(input.parse::<Expr>()?)
        } else {
            None
        };

        Ok(MacroArgs {
            string_arg,
            enum_arg,
            third_arg,
        })
    }
}

impl ToTokens for MacroArgs {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let string_arg = &self.string_arg;
        let enum_arg = &self.enum_arg;
        let third_arg = &self.third_arg;

        tokens.extend(quote! {
            // Handle the optional string argument
            {
                if let Some(string) = #string_arg {
                    println!("First argument (string): {}", string);
                } else {
                    println!("First argument (string): <not provided>");
                }
            }

            // Required enum argument
            println!("Second argument (enum): {}", stringify!(#enum_arg));

            // Handle the optional third argument
            {
                if let Some(expr) = #third_arg {
                    println!("Third argument: {:?}", expr);
                } else {
                    println!("Third argument: <not provided>");
                }
            }
        });
    }
}


#[proc_macro_attribute]
pub fn task(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the macro arguments
    let args = parse_macro_input!(attr as MacroArgs);

    // Parse the annotated function
    let input = parse_macro_input!(item as syn::ItemFn);

    // Extract the function name and body
    let fn_name = &input.sig.ident;
    let fn_body = &input.block;

    // Handle optional arguments
    let string_output = if let Some(ref string) = args.string_arg {
        quote! {
            println!("First argument (string): {}", #string);
        }
    } else {
        quote! {
            println!("First argument (string): <not provided>");
        }
    };

    let third_output = if let Some(ref expr) = args.third_arg {
        quote! {
            let result = #expr;
            println!("Third argument result: {:?}", result);
        }
    } else {
        quote! {
            println!("Third argument: <not provided>");
        }
    };

    // Generate the output code
    let output = quote! {
        // Original function definition
        #input

        // Add the macro logic
        fn #fn_name() {
            #string_output
            println!("Second argument (enum): {}", stringify!(#args.enum_arg));
            #third_output

            // Call the original function
            #fn_body
        }
    };

    TokenStream::from(output)
}

#[proc_macro]
pub fn add_entity(_item: TokenStream) -> TokenStream {
    _item
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

#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
