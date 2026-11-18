use crate::base::{Event, Rect, Size, Widget};
use crate::render::Painter;
use crate::style::Color;

#[derive(Clone, Copy, Debug, PartialEq)]
enum State {
    Normal,
    Hovered,
    Pressed,
}

#[derive(Clone, Copy, Debug)]
pub struct ButtonStyle {
    pub normal: Color,
    pub hovered: Color,
    pub pressed: Color,
}

pub struct Button {
    size: Size,
    style: ButtonStyle,
    state: State,
    on_click: Option<Box<dyn FnMut()>>,
}

impl Button {
    pub fn new(size: Size, style: ButtonStyle) -> Self {
        Self {
            size,
            style,
            state: State::Normal,
            on_click: None,
        }
    }

    pub fn on_click(mut self, handler: impl FnMut() + 'static) -> Self {
       self.on_click = Some(Box::new(handler));
       self 
    }

    fn set_state(&mut self, state: State) -> bool {
        let changed = self.state != state;
        self.state = state;
        changed
    }
}

impl Widget for Button {
    fn measure(&self, _available: Size) -> Size {
        self.size
    }

    fn paint(&self, bounds: Rect, painter: &mut dyn Painter) {
        let color = match self.state {
            State::Normal => self.style.normal,
            State::Hovered => self.style.hovered,
            State::Pressed => self.style.pressed,
        };
        painter.fill_rect(bounds, color);
    }

    fn event(&mut self, bounds: Rect, event: &Event) -> bool {
        match *event {
            Event::PointerMoved(point) => {
                if self.state == State::Pressed {
                    return false;
                }
                self.set_state(if bounds.contains(point) {
                    State::Hovered
                } else {
                    State::Normal
                })
            }
            Event::PointerPressed(point) if bounds.contains(point) => {
                self.set_state(State::Pressed)
            }
            Event::PointerReleased(point) => {
                let inside = bounds.contains(point);
                if inside && self.state == State::Pressed {
                    if let Some(handler) = self.on_click.as_mut() {
                        handler();
                    }
                }
                self.set_state(if inside { State::Hovered } else { State::Normal })
            }
            Event::PointerLeft => self.set_state(State::Normal),
            _ => false,
        }
    }
}