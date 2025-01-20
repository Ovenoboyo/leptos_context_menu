use std::sync::{Arc, MutexGuard};

use leptos::prelude::{ReadSignal, RwSignal, Set};
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

pub trait ContextMenuData<T>
where
    T: Send + Sync,
{
    fn get_menu_items(&self) -> ReadSignal<ContextMenuItems<T>>;
}

pub type ContextMenuItems<T> = Vec<ContextMenuItemInner<T>>;
pub type ContextMenuHandler<T> =
    Option<Arc<Box<dyn Fn(leptos::web_sys::MouseEvent, MutexGuard<'_, T>) + Send + Sync>>>;

pub struct ContextMenuItemInner<T>
where
    T: Send + Sync,
{
    pub key: String,
    pub name: String,
    pub handler: ContextMenuHandler<T>,
    pub children: Option<ContextMenuItems<T>>,
}

impl<T> Eq for ContextMenuItemInner<T> where T: Send + Sync {}

impl<T> PartialEq for ContextMenuItemInner<T>
where
    T: Send + Sync,
{
    fn eq(&self, other: &Self) -> bool {
        self.key.eq(&other.key)
    }
}

impl<T> ContextMenuItemInner<T>
where
    T: Send + Sync,
{
    pub fn new_with_handler(
        name: String,
        handler: impl Fn(leptos::web_sys::MouseEvent, MutexGuard<'_, T>) + 'static + Send + Sync,
        children: Option<ContextMenuItems<T>>,
    ) -> Self {
        ContextMenuItemInner {
            key: Uuid::new_v4().to_string(),
            name,
            handler: Some(Arc::new(Box::new(handler))),
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

impl<T> Clone for ContextMenuItemInner<T>
where
    T: Send + Sync,
{
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
    T: ContextMenuData<T> + 'static + Send + Sync,
{
    fn get_data(&self) -> MutexGuard<'_, T>;
    fn hide(&self);
    fn show(&self, mouse_event: leptos::ev::MouseEvent);
}
