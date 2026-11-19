use std::cell::RefCell;

thread_local! {
    static OUTGOING: RefCell<Option<String>> = const { RefCell::new(None) };
}

pub fn write(text: String) {
    OUTGOING.with(|slot| *slot.borrow_mut() = Some(text));
}

pub fn take_write() -> Option<String> {
    OUTGOING.with(|slot| slot.borrow_mut().take())
}
