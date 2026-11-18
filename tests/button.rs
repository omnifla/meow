use std::cell::Cell;
use std::rc::Rc;

use meow::base::{Event, Point, Rect, Size, Widget};
use meow::style::Color;
use meow::widgets::{Button, ButtonStyle};

const INSIDE: Point = Point { x: 10.0, y: 10.0 };
const OUTSIDE: Point = Point { x: 500.0, y: 500.0 };

fn setup() -> (Button, Rc<Cell<u32>>, Rect) {
    let color = Color::rgba(0.0, 0.0, 0.0, 1.0);
    let style = ButtonStyle {
        normal: color,
        hovered: color,
        pressed: color,
    };
    let clicks = Rc::new(Cell::new(0));
    let counter = Rc::clone(&clicks);
    let button = Button::new(
        Size {
            width: 100.0,
            height: 40.0,
        },
        style,
    )
    .on_click(move || counter.set(counter.get() + 1));
    (button, clicks, Rect::new(0.0, 0.0, 100.0, 40.0))
}

#[test]
fn press_and_release_inside_clicks_once() {
    let (mut button, clicks, bounds) = setup();
    button.event(bounds, &Event::PointerPressed(INSIDE));
    button.event(bounds, &Event::PointerReleased(INSIDE));
    assert_eq!(clicks.get(), 1);
}

#[test]
fn release_outside_does_not_click() {
    let (mut button, clicks, bounds) = setup();
    button.event(bounds, &Event::PointerPressed(INSIDE));
    button.event(bounds, &Event::PointerReleased(OUTSIDE));
    assert_eq!(clicks.get(), 0);
}

#[test]
fn press_outside_does_not_click() {
    let (mut button, clicks, bounds) = setup();
    button.event(bounds, &Event::PointerPressed(OUTSIDE));
    button.event(bounds, &Event::PointerReleased(INSIDE));
    assert_eq!(clicks.get(), 0);
}

#[test]
fn hover_reports_only_real_changes() {
    let (mut button, _, bounds) = setup();
    assert!(button.event(bounds, &Event::PointerMoved(INSIDE)));
    assert!(!button.event(bounds, &Event::PointerMoved(INSIDE)));
    assert!(button.event(bounds, &Event::PointerMoved(OUTSIDE)));
}
