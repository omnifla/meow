use super::{Event, Rect, Size};
use crate::render::Painter;

pub trait Widget {
    fn measure(&self, available: Size) -> Size;
    fn paint(&self, bounds: Rect, painter: &mut dyn Painter);

    fn event(&mut self, _bounds: Rect, _event: &Event) -> bool {
        false
    }
}
