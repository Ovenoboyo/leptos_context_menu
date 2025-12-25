# leptos_context_menu

A library for generating context menus in Leptos. You can populate the menu options dynamically.

## Installation

Run this command:

```bash
cargo add leptos_context_menu

```

Or add this to your `Cargo.toml`:

```toml
[dependencies]
leptos_context_menu = "0.1.0"

```

## Usage

To use this library, you must define a struct for your menu data, implement the `ContextMenuData` trait, and then attach the menu to an event listener.

```rust
use leptos::prelude::*;
use leptos_context_menu::{
    provide_context_menu_state, BottomSheet, ContextMenuData, ContextMenuItemInner,
    ContextMenuItems, Menu,
};

#[derive(Clone, Copy)]
struct MyMenuData {
    id: u32,
}

impl ContextMenuData<Self> for MyMenuData {
    fn get_menu_items(&self) -> ReadSignal<ContextMenuItems<Self>> {
        RwSignal::new(vec![
            // A simple menu item with a click handler
            ContextMenuItemInner::<Self>::new_with_handler(
                "Print Hello".to_string(),
                |_, data| {
                    leptos::logging::log!("Hello from ID: {}", data.id);
                },
                None, // No sub-menu items
            ),
            // A menu item with a nested sub-menu
            ContextMenuItemInner::<Self>::new_with_handler(
                "Nested Options".to_string(),
                |_, _| {}, 
                Some(vec![
                    ContextMenuItemInner::new_with_handler(
                        "Sub Item 1".to_string(),
                        |_, _| leptos::logging::log!("Clicked sub item"),
                        None,
                    )
                ]),
            ),
        ])
        .read_only()
    }
}

#[component]
fn App() -> impl IntoView {
    // 1. Initialize the global state
    provide_context_menu_state();

    // 2. Create the menu data and signal
    let menu_data = MyMenuData { id: 10 };
    let context_menu = create_rw_signal(BottomSheet::new(menu_data));

    // 3. Attach the menu to the window contextmenu event (right-click)
    window_event_listener(leptos::ev::contextmenu, move |ev| {
        ev.prevent_default(); // Prevent the browser's default menu
        context_menu.get().show(ev);
    });

    view! {
        <div style="height: 100vh;">
            "Right click anywhere on this screen."
        </div>
    }
}

```

## Styling

This library does not provide any CSS.

The menu will render as raw HTML elements. You must provide your own CSS to style the menu, position it correctly, and handle visibility states.
