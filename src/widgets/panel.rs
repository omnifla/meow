use crate::base::{Rect, Size, Widget};
use crate::render::Painter;
use crate::style::Color;

pub struct Panel {
    pub size: Size,
    pub color: Color,
}

impl Widget for Panel {
    fn measure(&self, _available: Size) -> Size {
        self.size
    }

    fn paint(&self, bounds: Rect, painter: &mut dyn Painter) {
        painter.fill_rect(bounds, self.color);
    }
}
