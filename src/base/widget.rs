use super::{Rect, Size};
use crate::render::Painter;

pub trait Widget {
    fn measure(&self, available: Size) -> Size;
    fn paint(&self, bounds: Rect, painter: &mut dyn Painter);
}
