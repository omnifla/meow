use std::cell::{Cell, RefCell};
use std::rc::Rc;

thread_local! {
    static DIRTY: Cell<bool> = const { Cell::new(false) };
}

fn mark_dirty() {
    DIRTY.with(|dirty| dirty.set(true));
}

pub fn take_dirty() -> bool {
    DIRTY.with(|dirty| dirty.replace(false))
}

pub struct Signal<T> {
    value: Rc<RefCell<T>>,
}

impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Self {
            value: Rc::clone(&self.value),
        }
    }
}

impl<T> Signal<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: Rc::new(RefCell::new(value)),
        }
    }

    pub fn set(&self, value: T) {
        *self.value.borrow_mut() = value;
        mark_dirty();
    }

    pub fn update(&self, change: impl FnOnce(&mut T)) {
        change(&mut self.value.borrow_mut());
        mark_dirty();
    }

    pub fn with<R>(&self, read: impl FnOnce(&T) -> R) -> R {
        read(&self.value.borrow())
    }
}

impl<T: Clone> Signal<T> {
    pub fn get(&self) -> T {
        self.value.borrow().clone()
    }
}
