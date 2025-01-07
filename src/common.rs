use std::{rc::Rc, sync::MutexGuard};

use leptos::{RwSignal, SignalSet};
use uuid::Uuid;

#[derive(Default)]
pub struct ContextMenuState {
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

pub trait ContextMenuData<T> {
    fn get_menu_items(&self) -> ContextMenuItems<T>;
}

pub type ContextMenuItems<T> = Vec<ContextMenuItemInner<T>>;
pub type ContextMenuHandler<T> =
    Option<Rc<Box<dyn Fn(leptos::web_sys::MouseEvent, MutexGuard<'_, T>)>>>;

pub struct ContextMenuItemInner<T> {
    pub key: String,
    pub name: String,
    pub handler: ContextMenuHandler<T>,
    pub children: Option<ContextMenuItems<T>>,
}

impl<T> ContextMenuItemInner<T> {
    pub fn new_with_handler(
        name: String,
        handler: impl Fn(leptos::web_sys::MouseEvent, MutexGuard<'_, T>) + 'static,
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

pub trait Menu<T>
where
    T: ContextMenuData<T> + 'static,
{
    fn get_data(&self) -> MutexGuard<'_, T>;
    fn hide(&self);
    fn show(&self, mouse_event: leptos::ev::MouseEvent);
}
