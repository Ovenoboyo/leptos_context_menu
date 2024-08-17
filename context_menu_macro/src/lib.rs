extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::quote;
use syn::{parse_macro_input, ItemImpl, Lit, Meta, MetaList};

#[proc_macro_attribute]
pub fn context_menu_attr(_args: TokenStream, input: TokenStream) -> TokenStream {
    eprintln!("here");
    // Simply return the input as we're going to parse everything in the `impl_context_menu` macro
    input
}

#[proc_macro_attribute]
pub fn context_menu(_attr: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let item_impl = parse_macro_input!(input as ItemImpl);

    // Extract the type of the struct the impl block is for
    let struct_ident = &item_impl.self_ty;

    // Initialize tokens to hold generated code
    // let mut methods = Vec::new();

    // Iterate over each item in the impl block
    for item in item_impl.items.iter() {
        if let syn::ImplItem::Fn(method) = item {
            let method_name = &method.sig.ident;

            // Check for the `context_menu` attribute
            for attr in &method.attrs {
                if attr.path().is_ident("context_menu_attr") {
                    if let Meta::List(meta_list) = &attr.meta {
                        let tokens = meta_list.tokens;
                        for token in tokens {
                            if let TokenTree::Ident(ident) = token {
                                // Will implement some day
                            }
                        }
                    }
                }

                // if let Ok(meta) = attr.parse_meta() {}
            }
        }
    }

    // Combine all the generated code
    let expanded = quote! {
        // #(#methods)*
    };

    // Return the generated implementation as a TokenStream
    TokenStream::from(expanded)
}
