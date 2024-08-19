use std::time::Duration;

use leptos::{
    create_rw_signal, logging::log, mount_to_body, set_interval, spawn_local, view, SignalGet,
};
use leptos_context_menu::{
    provide_context_menu_state, ContextMenu, ContextMenuData, ContextMenuItemInner,
    ContextMenuItems,
};

#[derive(Clone, Copy)]
struct DataContextMenu {
    string_data: u32,
}

// #[context_menu]
impl DataContextMenu {
    // #[context_menu_attr(name = "hello")]
    fn hello(&self) {
        log!("{}", self.string_data);
    }

    // #[context_menu_attr(name="hello1", children=[bye, bye1])]
    fn hello1(&self) {}

    // #[context_menu_attr(name = "bye")]
    fn bye(&self) {}

    // #[context_menu_attr(name = "bye1")]
    fn bye1(&self) {}
}

impl ContextMenuData<Self> for DataContextMenu {
    fn get_menu_items(&self) -> ContextMenuItems<Self> {
        vec![
            ContextMenuItemInner::new_with_handler(
                "Item 1".to_string(),
                |cx| {
                    cx.hello();
                },
                Some(vec![ContextMenuItemInner::new_with_handler(
                    "Item 1.1".to_string(),
                    |cx| {
                        cx.hello1();
                    },
                    Some(vec![ContextMenuItemInner::new_with_handler(
                        "Item 1.1.1".to_string(),
                        |cx| {
                            cx.bye1();
                        },
                        None,
                    )]),
                )]),
            ),
            ContextMenuItemInner::new_with_handler(
                "Item 2".to_string(),
                |cx| {
                    cx.bye();
                },
                None,
            ),
        ]
    }
}

fn main() {
    console_error_panic_hook::set_once();

    // Optional if you only want one context menu on the screen at a time
    provide_context_menu_state();

    let context_menu_data = DataContextMenu { string_data: 0 };

    let context_menu = create_rw_signal(ContextMenu::new(context_menu_data));

    set_interval(
        move || {
            let binding = context_menu.get();
            let mut context_menu_data = binding.get_data();
            context_menu_data.string_data += 1;
            if context_menu_data.string_data > 10000 {
                context_menu_data.string_data = 0;
            }
            log!("{}", context_menu_data.string_data);
        },
        Duration::from_millis(1000),
    );

    mount_to_body(move || {
        leptos::window_event_listener(leptos::ev::contextmenu, move |ev| {
            ev.prevent_default();
            context_menu.get().show(ev);
        });
        view! { <div style="height: 100vh;"></div> }
    });
}
