use std::sync::{Arc, Mutex, MutexGuard};

use leptos::{
    ev::{mousedown, mousemove, mouseup, touchend, touchmove, touchstart, transitionend},
    html::Div,
    mount::mount_to_body,
    prelude::*,
    view, IntoView,
};
use leptos_use::{use_event_listener, use_event_listener_with_options, UseEventListenerOptions};

use crate::{
    common::ContextMenuState, ContextMenuData, ContextMenuItemInner, ContextMenuItems, Menu,
};

struct RenderMenuArgs<T>
where
    T: ContextMenuData<T> + 'static + Send + Sync,
{
    ctx: Arc<Mutex<T>>,
    items: ReadSignal<ContextMenuItems<T>>,
    show: RwSignal<bool>,
    owner: Owner,
}

#[derive(Clone)]
pub struct BottomSheet<T>
where
    T: ContextMenuData<T> + 'static + Send + Sync,
{
    data: Arc<Mutex<T>>,
    show_signal: RwSignal<bool>,
    root_items: RwSignal<ReadSignal<ContextMenuItems<T>>>,
    owner: Owner,
}

impl<T> BottomSheet<T>
where
    T: ContextMenuData<T> + 'static + Send + Sync,
{
    pub fn new(data: T) -> Self {
        let ctx = Self {
            data: Arc::new(Mutex::new(data)),
            show_signal: RwSignal::new(false),
            root_items: RwSignal::new(RwSignal::new(Default::default()).read_only()),
            owner: Owner::new(),
        };
        ctx.render_root_view();

        if let Some(context_menu_state) = use_context::<RwSignal<ContextMenuState>>() {
            context_menu_state.update(|c| c.add_menu(ctx.show_signal));
        }

        ctx
    }

    fn render_root_view(&self) {
        let show = self.show_signal;
        let root_node_ref = NodeRef::new();
        let root_items = self.root_items.get();
        let data = self.data.clone();
        let owner = self.owner.clone();

        let view = view! {
            <div
                class="context-menu-root"
                node_ref=root_node_ref
                style:display=move || if show.get() { "block" } else { "none" }
            >

                {move || {
                    let owner = owner.clone();
                    render_menu(RenderMenuArgs {
                            ctx: data.clone(),
                            items: root_items,
                            show,
                            owner,
                        })
                        .into_any()
                }}
            </div>
        };

        mount_to_body(move || view);
    }
}

impl<T> Menu<T> for BottomSheet<T>
where
    T: ContextMenuData<T> + 'static + Send + Sync,
{
    fn hide(&self) {
        self.show_signal.set(false);
    }

    fn show(&self, _: leptos::ev::MouseEvent) {
        let ctx = self.data.lock().unwrap();
        self.root_items
            .set(self.owner.with(|| ctx.get_menu_items()));
        drop(ctx);

        if let Some(context_menu_state) = use_context::<RwSignal<ContextMenuState>>() {
            context_menu_state.update(|c| c.hide_all());
        }
        self.show_signal.set(true);
    }

    fn get_data(&self) -> MutexGuard<'_, T> {
        self.data.lock().unwrap()
    }
}

fn flatten<T>(item: &ContextMenuItemInner<T>) -> Vec<ContextMenuItemInner<T>>
where
    T: ContextMenuData<T> + 'static + Send + Sync,
{
    let mut flattened_children = vec![];
    flattened_children.push(item.clone());
    if let Some(children) = &item.children {
        for child in children {
            let children = flatten(child);
            flattened_children.extend(children.into_iter());
        }
    }
    flattened_children
}

fn render_menu<T>(args: RenderMenuArgs<T>) -> impl IntoView
where
    T: ContextMenuData<T> + 'static + Send + Sync,
{
    let node_ref = NodeRef::<Div>::new();
    let root_node_ref = NodeRef::<Div>::new();

    let is_dragging = RwSignal::new(false);
    let has_moved = RwSignal::new(false);
    let start_offset = RwSignal::new(0);
    let page_height = RwSignal::new(0);
    let drag_up = RwSignal::new(None);
    let bottom_sheet_pos = RwSignal::new(0);

    let listener = move |client_y: i32| {
        is_dragging.set(true);
        start_offset.set(client_y);
        page_height.set(document().body().unwrap().client_height());
        has_moved.set(false);
        if let Some(el) = node_ref.get() {
            let _ = el.style(("transition", "all 0s"));
        }
    };
    let _ = use_event_listener(node_ref, mousedown, move |ev| {
        listener(ev.client_y());
    });
    let _ = use_event_listener(node_ref, touchstart, move |ev| {
        let touch = ev.touches().get(0).unwrap();
        listener(touch.client_y());
    });

    let listener = move || {
        if is_dragging.get_untracked() {
            is_dragging.set(false);

            if let Some(el) = node_ref.get() {
                if let Some(drag_up) = drag_up.get() {
                    if drag_up {
                        let sixty_perc = -((0.6) * page_height.get() as f64) as i32;
                        let _ = el.style(("transform", format!("translateY({}px)", sixty_perc)));
                        let _ = el.style(("transition", "all 0.2s"));
                        bottom_sheet_pos.set(sixty_perc);
                    } else {
                        let transition_time = if bottom_sheet_pos.get() < 0 {
                            "0.4s"
                        } else {
                            "0.2s"
                        };
                        let _ = el.style((
                            "transform",
                            format!(
                                "translateY({}px)",
                                ((1f64 - 0.6) * page_height.get() as f64)
                            ),
                        ));
                        let _ = el.style(("transition", format!("all {}", transition_time)));

                        if let Some(el) = root_node_ref.get() {
                            let _ = el.style(("opacity", "0"));
                            let _ = el.style(("transition", format!("all {}", transition_time)));
                        }

                        let show = args.show;
                        let _ = use_event_listener_with_options(
                            node_ref,
                            transitionend,
                            move |_| {
                                show.set(false);
                            },
                            UseEventListenerOptions::default().once(true),
                        );
                    }
                }
            }
        }
    };

    let _ = use_event_listener(node_ref, mouseup, move |_| listener());

    let _ = use_event_listener(node_ref, touchend, move |_| listener());

    Effect::new(move || {
        let _ = args.show.get();
        drag_up.set(None);
        is_dragging.set(false);
        start_offset.set(0);
        bottom_sheet_pos.set(0);
        has_moved.set(false);

        if let Some(elem) = node_ref.get() {
            let _ = elem.style(("transform", "unset"));
            let _ = elem.style(("transition", "unset"));
        }

        if let Some(elem) = root_node_ref.get() {
            let _ = elem.style(("opacity", "unset"));
        }
    });

    let listener = move |client_y: i32| {
        if is_dragging.get() {
            let page_height = page_height.get();
            let start_offset = start_offset.get();
            let bottom_sheet_pos = bottom_sheet_pos.get();
            let sixty_perc = -((0.6) * page_height as f64) as i32;
            let fourty_perc = ((1f64 - 0.6) * page_height as f64) as i32;
            let client_y_diff =
                (bottom_sheet_pos + client_y - start_offset).clamp(sixty_perc, fourty_perc);

            if client_y_diff - bottom_sheet_pos < -20 {
                drag_up.set(Some(true));
            } else {
                drag_up.set(Some(false));
            }

            if client_y_diff.abs() > 5 {
                has_moved.set(true);
            }

            if let Some(elem) = node_ref.get() {
                let _ = elem.style(("transform", format!("translateY({}px)", client_y_diff)));
            }
        }
    };
    let _ = use_event_listener(document().body(), mousemove, move |ev| {
        listener(ev.client_y());
    });
    let _ = use_event_listener(document().body(), touchmove, move |ev| {
        let touch = ev.touches().get(0).unwrap();
        listener(touch.client_y());
    });

    let flattened_children = Memo::new(move |_| {
        let mut flattened_children = vec![];
        for item in args.items.get() {
            flattened_children.extend(flatten(&item).into_iter());
        }
        flattened_children
    });

    view! {
        <div
            node_ref=root_node_ref
            class="context-menu-outer"
            style="position: fixed; width: 100vw; height: 100vh; background-color: rgba(0, 0, 0, 1); bottom: 0; left: 0;"
            style:display=move || if args.show.get() { "block" } else { "none" }
            on:click=move |_| { args.show.set(false) }
        >

            <div
                node_ref=node_ref
                class="bottom-sheet-container"
                style="position: absolute; left: 0; top: 60vh; background-color: blue; width: 100vw; height: 100vh;"
                on:click=move |ev| {
                    ev.stop_propagation();
                }
            >
                <For
                    each=move || flattened_children.get()
                    key=move |item| item.key.clone()
                    children=move |item| {
                        let item_name = item.name.clone();
                        let item_children = item.children.clone();
                        let item_handler = item.handler.clone();
                        let ctx = args.ctx.clone();
                        let show = args.show;
                        let owner = args.owner.child();
                        view! {
                            <div
                                class="context-menu-item context-menu-open"
                                style="display: flex; align-items: center;"
                                on:click=move |e| {
                                    if !has_moved.get_untracked() {
                                        if let Some(handler) = item_handler.clone() {
                                            owner
                                                .with(|| {
                                                    let ctx = ctx.lock().unwrap();
                                                    handler(e, ctx)
                                                });
                                        }
                                        show.set(false);
                                    }
                                }
                            >
                                <div
                                    class="context-menu-item-text"
                                    style="overflow: hidden; text-overflow: ellipsis;"
                                >
                                    {item_name.clone()}
                                </div>

                                <div
                                    class="context-menu-item-icon"
                                    style="min-width: 14px; min-height: 14px; width: 14px; height: 14px; display: flex;"
                                >

                                    {move || {
                                        if item_children.is_some() {
                                            view! {
                                                <svg
                                                    class="context-menu-right-arrow"
                                                    viewBox="0 0 1024 1024"
                                                >
                                                    <path d="M307.018 49.445c11.517 0 23.032 4.394 31.819 13.18L756.404 480.18c8.439 8.438 13.181 19.885 13.181 31.82s-4.741 23.38-13.181 31.82L338.838 961.376c-17.574 17.573-46.065 17.573-63.64-0.001-17.573-17.573-17.573-46.065 0.001-63.64L660.944 512 275.198 126.265c-17.574-17.573-17.574-46.066-0.001-63.64C283.985 53.839 295.501 49.445 307.018 49.445z"></path>
                                                </svg>
                                            }
                                                .into_any()
                                        } else {
                                            ().into_any()
                                        }
                                    }}

                                </div>

                            </div>
                        }
                    }
                />
            </div>
        </div>
    }
}
