extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Meta};

#[proc_macro_derive(ContextMenu)]
pub fn derive_context_menu(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Get the struct's name
    let name = input.ident;

    // Extract the function name with the #[context_menu] attribute
    let mut context_menu_fn = None;

    for attr in input.attrs {
        if attr.path().is_ident("context_menu") {
            if let Meta::List(path) = attr.meta {
                for token in path.tokens {
                    if let TokenTree::Ident(ident) = token {
                        context_menu_fn = Some(ident);
                    }
                }
            }
        }
    }

    let context_menu_items = if let Some(fn_name) = context_menu_fn {
        quote! {
            self.#fn_name()
        }
    } else {
        quote! {
            ContextMenuItems::default()
        }
    };

    // Generate the implementation of the trait
    let expanded = quote! {
        impl ContextMenu for #name {
            fn show(&self, mouse_event: leptos::ev::MouseEvent) {
                use leptos::{For, html::div, IntoView};
                use leptos_context_menu::{HtmlElement, on_click_outside, JsCast, ContextMenuItems};

                let x = mouse_event.client_x();
                let y = mouse_event.client_y();

                let items: ContextMenuItems = #context_menu_items;

                let view = move || {
                    let node_ref = leptos::create_node_ref();

                    let _ = on_click_outside(node_ref, move |_| {
                        let div = node_ref.get_untracked();
                        if let Some(div) = div {
                            div.remove();
                        }
                    });

                    view! {
                        <div node_ref=node_ref class="context-menu-outer" style={format!("position: fixed; max-width: 600px; min-width: 100px; border: 1px solid; z-index: 100; top: {}px; left: {}px", y, x)}>
                            <For each=move || items.clone() key=move |i| i.key.clone() children=move |item| {
                                view! {
                                    <div class="context-menu-item" style="display: flex; align-items: center;">
                                        <div class="context-menu-item-text" style="overflow: hidden; text-overflow: ellipsis;">{item.name}</div>
                                        {
                                            move || {
                                                if item.children.is_some() {
                                                    view! {
                                                        <div class="context-menu-item-icon" style="min-width: 14px; min-height: 14px; width: 14px; height: 14px; display: flex;">
                                                            <svg class="context-menu-right-arrow" aria-hidden="true" viewBox="0 0 1024 1024"><path d="M307.018 49.445c11.517 0 23.032 4.394 31.819 13.18L756.404 480.18c8.439 8.438 13.181 19.885 13.181 31.82s-4.741 23.38-13.181 31.82L338.838 961.376c-17.574 17.573-46.065 17.573-63.64-0.001-17.573-17.573-17.573-46.065 0.001-63.64L660.944 512 275.198 126.265c-17.574-17.573-17.574-46.066-0.001-63.64C283.985 53.839 295.501 49.445 307.018 49.445z"></path></svg>
                                                        </div>
                                                    }.into_view()
                                                } else {
                                                    view !{}.into_view()
                                                }
                                            }
                                        }
                                    </div>
                                }
                            } />
                        </div>
                    }
                };
                let mut element = leptos::document().get_element_by_id("context-menu");
                if element.is_none() {
                    let div = div();
                    div.set_id("context-menu");

                    leptos::mount_to_body(move || div);
                    element = leptos::document().get_element_by_id("context-menu");
                }
                let element = element.unwrap();
                if element.has_child_nodes() {
                    element.set_inner_html("");
                }
                let element: HtmlElement = element.unchecked_into();
                leptos::mount_to(element, view);
            }
        }
    };

    // Convert the generated code into a TokenStream and return it
    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn context_menu(_: TokenStream, item: TokenStream) -> TokenStream {
    item
}
