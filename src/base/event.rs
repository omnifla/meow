use super::Point;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
    PointerMoved(Point),
    PointerPressed(Point),
    PointerReleased(Point),
    PointerLeft,
}
