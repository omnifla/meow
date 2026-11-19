use std::cell::{Cell, RefCell};
use std::rc::Rc;

use meow::base::{Event, Point, Rect, Size, Widget};
use meow::render::Painter;
use meow::style::Color;
use meow::widgets::Scroll;

const INSIDE: Point = Point { x: 20.0, y: 20.0 };
const OUTSIDE: Point = Point { x: 500.0, y: 500.0 };

struct Null;

impl Painter for Null {
    fn fill_rect(&mut self, _rect: Rect, _color: Color) {}
}

struct Probe {
    height: f32,
    painted: Rc<Cell<Rect>>,
    events: Rc<RefCell<Vec<Event>>>,
}

impl Widget for Probe {
    fn measure(&self, available: Size) -> Size {
        Size {
            width: available.width,
            height: self.height,
        }
    }

    fn paint(&self, bounds: Rect, _painter: &mut dyn Painter) {
        self.painted.set(bounds);
    }

    fn event(&mut self, _bounds: Rect, event: &Event) -> bool {
        self.events.borrow_mut().push(event.clone());
        false
    }
}

fn setup(content: f32) -> (Scroll, Rc<Cell<Rect>>, Rc<RefCell<Vec<Event>>>, Rect) {
    let painted = Rc::new(Cell::new(Rect::default()));
    let events = Rc::new(RefCell::new(Vec::new()));
    let probe = Probe {
        height: content,
        painted: Rc::clone(&painted),
        events: Rc::clone(&events),
    };
    let scroll = Scroll::new(probe).height(100.0);
    (scroll, painted, events, Rect::new(0.0, 0.0, 210.0, 100.0))
}

#[test]
fn short_content_does_not_scroll() {
    let (mut scroll, painted, _, bounds) = setup(50.0);
    assert!(!scroll.event(bounds, &Event::Scroll(INSIDE, 30.0)));
    scroll.paint(bounds, &mut Null);
    assert_eq!(painted.get(), Rect::new(0.0, 0.0, 200.0, 50.0));
}

#[test]
fn scrolling_moves_content_up() {
    let (mut scroll, painted, _, bounds) = setup(300.0);
    assert!(scroll.event(bounds, &Event::Scroll(INSIDE, 50.0)));
    scroll.paint(bounds, &mut Null);
    assert_eq!(painted.get(), Rect::new(0.0, -50.0, 200.0, 300.0));
}

#[test]
fn scrolling_clamps_at_the_end() {
    let (mut scroll, painted, _, bounds) = setup(300.0);
    scroll.event(bounds, &Event::Scroll(INSIDE, 1000.0));
    scroll.paint(bounds, &mut Null);
    assert_eq!(painted.get().origin.y, -200.0);
}

#[test]
fn scrolling_clamps_at_the_start() {
    let (mut scroll, painted, _, bounds) = setup(300.0);
    assert!(!scroll.event(bounds, &Event::Scroll(INSIDE, -30.0)));
    scroll.paint(bounds, &mut Null);
    assert_eq!(painted.get().origin.y, 0.0);
}

#[test]
fn wheel_outside_the_viewport_is_ignored() {
    let (mut scroll, painted, _, bounds) = setup(300.0);
    assert!(!scroll.event(bounds, &Event::Scroll(OUTSIDE, 50.0)));
    scroll.paint(bounds, &mut Null);
    assert_eq!(painted.get().origin.y, 0.0);
}

#[test]
fn pointer_outside_the_viewport_reaches_children_as_far_away() {
    let (mut scroll, _, events, bounds) = setup(300.0);
    scroll.event(bounds, &Event::PointerPressed(OUTSIDE));
    scroll.event(bounds, &Event::PointerPressed(INSIDE));
    let far = Point {
        x: f32::NEG_INFINITY,
        y: f32::NEG_INFINITY,
    };
    assert_eq!(
        *events.borrow(),
        vec![Event::PointerPressed(far), Event::PointerPressed(INSIDE)]
    );
}