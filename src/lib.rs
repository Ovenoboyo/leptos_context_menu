use leptos::html::Div;
use leptos::logging::log;
use leptos::{
    create_effect, create_node_ref, create_rw_signal, document, provide_context, use_context, view,
    CollectView, NodeRef, RwSignal, SignalGetUntracked, SignalSet,
};
use leptos::{For, IntoView, SignalGet, SignalUpdate};
use leptos_use::on_click_outside;
use std::rc::Rc;
use std::sync::{Mutex, MutexGuard};
use uuid::Uuid;

pub use context_menu_macro::{context_menu, context_menu_attr};

type HoverItems<T> = Vec<(String, ContextMenuItems<T>, i32, i32, NodeRef<Div>)>;

struct RenderMenuArgs<T>
where
    T: ContextMenuData<T> + 'static,
{
    ctx: Rc<Mutex<T>>,
    root_node_ref: NodeRef<Div>,
    hovered_items: RwSignal<HoverItems<T>>,
    items: ContextMenuItems<T>,
    x: i32,
    y: i32,
    level: usize,
    node_ref: NodeRef<Div>,
}

pub type ContextMenuItems<T> = Vec<ContextMenuItemInner<T>>;
pub type ContextMenuHandler<T> = Option<Rc<Box<dyn Fn(MutexGuard<'_, T>)>>>;

pub struct ContextMenuItemInner<T> {
    pub key: String,
    pub name: String,
    pub handler: ContextMenuHandler<T>,
    pub children: Option<ContextMenuItems<T>>,
}

pub trait ContextMenuData<T> {
    fn get_menu_items(&self) -> ContextMenuItems<T>;
}

#[derive(Default)]
struct ContextMenuState {
    context_menu_shows: Vec<RwSignal<bool>>,
}

impl ContextMenuState {
    pub fn hide_all(&self) {
        for show in &self.context_menu_shows {
            show.set(false);
        }
    }

    pub fn add_menu(&mut self, signal: RwSignal<bool>) {
        self.context_menu_shows.push(signal);
    }
}

pub struct ContextMenu<T>
where
    T: ContextMenuData<T> + 'static,
{
    hovered_items: RwSignal<HoverItems<T>>,
    ctx: Rc<Mutex<T>>,
    root_view: Mutex<Option<NodeRef<Div>>>,
    coords: RwSignal<(i32, i32)>,
    show: RwSignal<bool>,
}

impl<T> ContextMenu<T>
where
    T: ContextMenuData<T> + 'static,
{
    pub fn new(data: T) -> Self {
        let ctx = Self {
            ctx: Rc::new(Mutex::new(data)),
            hovered_items: create_rw_signal(Vec::new()),
            root_view: Mutex::new(None),
            coords: create_rw_signal((0, 0)),
            show: create_rw_signal(false),
        };

        ctx.render_root_view();

        if let Some(context_menu_state) = leptos::use_context::<RwSignal<ContextMenuState>>() {
            context_menu_state.update(|c| c.add_menu(ctx.show));
        }

        ctx
    }

    fn render_menu(args: RenderMenuArgs<T>) -> impl IntoView {
        let context_menu_bounds = move |level: usize, active_item_node_ref: NodeRef<Div>| {
            let width = if level == 0 {
                args.root_node_ref.get_untracked().unwrap().client_width()
            } else {
                let prev_item = args.hovered_items.get_untracked();
                let prev_item = prev_item.get(level - 1).unwrap();
                prev_item.4.get_untracked().unwrap().client_width()
            };

            let bounds = active_item_node_ref
                .get_untracked()
                .unwrap()
                .get_bounding_client_rect();
            (width, bounds.top() as i32 - 1)
        };

        let handle_hover = move |item: &ContextMenuItemInner<T>, x, y, level| {
            let has_children = item.children.is_some();
            let current_level = args.hovered_items.get_untracked().len();
            let binding = args.hovered_items.get_untracked();
            let last_item = binding.last();

            if has_children && current_level == level + 1 {
                if let Some(last_item) = last_item {
                    let last_item = last_item.0.clone();
                    if last_item == item.key {
                        return;
                    }
                }
            }

            if current_level > level {
                args.hovered_items.update(|hovered_items| {
                    hovered_items.truncate(level);
                });
            }
            if let Some(children) = item.children.clone() {
                let new_menu_node_ref = create_node_ref::<Div>();
                args.hovered_items.update(|hovered_items| {
                    hovered_items.push((item.key.clone(), children, x, y, new_menu_node_ref));
                });
            }
        };

        let y_pos = create_rw_signal(args.y);
        let x_pos = create_rw_signal(args.x);

        let node_ref = args.node_ref;

        create_effect(move |_| {
            let el = node_ref.get_untracked();
            if let Some(el) = el {
                let body_width = document().body().unwrap().client_width();
                let body_height = document().body().unwrap().client_height();

                let height = el.offset_height();
                let width = el.offset_width();

                if height + args.y > body_height {
                    y_pos.set(args.y - height);
                }

                if width + args.x > body_width {
                    let mut min_x = args
                        .root_node_ref
                        .get_untracked()
                        .unwrap()
                        .get_bounding_client_rect()
                        .left();
                    for (_, _, _, _, node_ref) in args.hovered_items.get_untracked() {
                        let left = node_ref
                            .get_untracked()
                            .unwrap()
                            .get_bounding_client_rect()
                            .left();
                        if left < min_x {
                            min_x = left;
                        }
                    }
                    x_pos.set(min_x as i32 - width);
                }
            }
        });

        view! {
            <div
                node_ref=node_ref
                class="context-menu-outer"
                style="position: fixed; max-width: 600px; box-sizing: border-box; border: 1px solid;  min-width: 100px"
                style:top=move || format!("{}px", y_pos.get())
                style:left=move || format!("{}px", x_pos.get())
            >
                <For
                    each=move || args.items.clone()
                    key=move |item| item.key.clone()
                    children=move |item| {
                        let item_name = item.name.clone();
                        let item_children = item.children.clone();
                        let item_handler = item.handler.clone();
                        let item_key = item.key.clone();
                        let active_item_node_ref = create_node_ref::<Div>();
                        let ctx = args.ctx.clone();
                        view! {
                            <div
                                class="context-menu-item"
                                class=(
                                    "context-menu-open",
                                    move || {
                                        let hovered_items = args.hovered_items.get();
                                        hovered_items.iter().any(|i| i.0 == item_key.clone())
                                    },
                                )
                                style="display: flex; align-items: center;"
                                node_ref=active_item_node_ref
                                on:mouseover=move |_| {
                                    let (width, page_y) = context_menu_bounds(
                                        args.level,
                                        active_item_node_ref,
                                    );
                                    handle_hover(&item, args.x + width, page_y, args.level)
                                }
                                on:click=move |_| {
                                    if let Some(handler) = item_handler.clone() {
                                        let ctx = ctx.lock().unwrap();
                                        handler(ctx);
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
                                                    aria-hidden="true"
                                                    viewBox="0 0 1024 1024"
                                                >
                                                    <path d="M307.018 49.445c11.517 0 23.032 4.394 31.819 13.18L756.404 480.18c8.439 8.438 13.181 19.885 13.181 31.82s-4.741 23.38-13.181 31.82L338.838 961.376c-17.574 17.573-46.065 17.573-63.64-0.001-17.573-17.573-17.573-46.065 0.001-63.64L660.944 512 275.198 126.265c-17.574-17.573-17.574-46.066-0.001-63.64C283.985 53.839 295.501 49.445 307.018 49.445z"></path>
                                                </svg>
                                            }
                                                .into_view()
                                        } else {
                                            view! {}.into_view()
                                        }
                                    }}

                                </div>

                            </div>
                        }
                    }
                />
            </div>
        }
    }

    pub fn render_root_view(&self) {
        let ctx = self.ctx.lock().unwrap();
        let root_items = ctx.get_menu_items().clone();
        drop(ctx);

        let ctx = self.ctx.clone();
        let root_node_ref = create_node_ref();
        let hovered_items = self.hovered_items;
        let coords = self.coords;
        let show = self.show;

        let view = view! {
            <div class="context-menu-root" node_ref=root_node_ref>
                {move || {
                    if show.get() {
                        let mut ret = vec![];
                        let (x, y) = coords.get();
                        let root_node_ref = create_node_ref();
                        ret.push(
                            Self::render_menu(RenderMenuArgs {
                                    ctx: ctx.clone(),
                                    root_node_ref,
                                    hovered_items,
                                    items: root_items.clone(),
                                    x,
                                    y,
                                    level: 0,
                                    node_ref: root_node_ref,
                                })
                                .into_view(),
                        );
                        let ctx = ctx.clone();
                        ret.push(
                            hovered_items
                                .get()
                                .iter()
                                .enumerate()
                                .map(|(level, (_, children, child_x, child_y, node_ref))| {
                                    Self::render_menu(RenderMenuArgs {
                                        ctx: ctx.clone(),
                                        root_node_ref,
                                        hovered_items,
                                        items: children.clone(),
                                        x: *child_x,
                                        y: *child_y,
                                        level: level + 1,
                                        node_ref: *node_ref,
                                    })
                                })
                                .collect_view(),
                        );
                        ret.collect_view()
                    } else {
                        view! {}.into_view()
                    }
                }}
            </div>
        };

        let _ = on_click_outside(root_node_ref, move |_| {
            show.set(false);
            hovered_items.set(vec![]);
        });

        leptos::mount_to_body(move || view);

        let mut element = self.root_view.lock().unwrap();
        *element = Some(root_node_ref);
    }

    pub fn show(&self, mouse_event: leptos::ev::MouseEvent) {
        let x = mouse_event.client_x();
        let y = mouse_event.client_y();

        self.hovered_items.set(vec![]);
        self.coords.set((x, y));

        if let Some(context_menu_state) = use_context::<RwSignal<ContextMenuState>>() {
            context_menu_state.update(|c| c.hide_all());
        }
        self.show.set(true);
    }
}

impl<T> Clone for ContextMenuItemInner<T> {
    fn clone(&self) -> Self {
        ContextMenuItemInner {
            key: self.key.clone(),
            name: self.name.clone(),
            handler: self.handler.clone(),
            children: self.children.clone(),
        }
    }
}

impl<T> ContextMenuItemInner<T> {
    pub fn new_with_handler(
        name: String,
        handler: impl Fn(MutexGuard<'_, T>) + 'static,
        children: Option<ContextMenuItems<T>>,
    ) -> Self {
        ContextMenuItemInner {
            key: Uuid::new_v4().to_string(),
            name,
            handler: Some(Rc::new(Box::new(handler))),
            children,
        }
    }

    pub fn new(name: String, children: Option<ContextMenuItems<T>>) -> Self {
        ContextMenuItemInner {
            key: Uuid::new_v4().to_string(),
            name,
            handler: None,
            children,
        }
    }
}

pub fn provide_context_menu_state() {
    provide_context(RwSignal::new(ContextMenuState::default()));
}
