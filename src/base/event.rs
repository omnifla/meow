use super::Point;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Backspace,
    Delete,
    Left,
    Right,
    Home,
    End,
    Enter,
    Escape,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    PointerMoved(Point),
    PointerPressed(Point),
    PointerReleased(Point),
    PointerLeft,
    Char(char),
    KeyPressed(Key, Modifiers),
    SelectAll,
    Copy,
    Cut,
    Paste(String),
    Scroll(Point, f32),
}
