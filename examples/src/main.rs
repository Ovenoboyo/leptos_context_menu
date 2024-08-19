use leptos::{logging::log, mount_to_body, view};
use leptos_context_menu::{
    ContextMenu, ContextMenuData, ContextMenuItemInner, ContextMenuItems,
};

struct DataContextMenu {
    string_data: String,
}

// #[context_menu]
impl DataContextMenu {
    // #[context_menu_attr(name = "hello")]
    fn hello(&self) {
        log!("Hello, World!");
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

    mount_to_body(move || {
        let context_menu = DataContextMenu {
            string_data: "Hello, World!".into(),
        };
        let context_menu: ContextMenu<DataContextMenu> = ContextMenu::new(context_menu);
        leptos::window_event_listener(leptos::ev::contextmenu, move |ev| {
            ev.prevent_default();
            context_menu.show(ev);
        });
        view! { <div style="height: 100vh;"></div> }
    });
}
