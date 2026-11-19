use std::cell::Cell;

use crate::base::{Event, Point, Rect, Size, Widget};
use crate::render::Painter;
use crate::style::Color;

const GUTTER: f32 = 10.0;
const THUMB_WIDTH: f32 = 4.0;
const THUMB_INSET: f32 = 3.0;
const MIN_THUMB: f32 = 24.0;
const AWAY: Point = Point {
    x: f32::NEG_INFINITY,
    y: f32::NEG_INFINITY,
};

pub struct Scroll {
    child: Box<dyn Widget>,
    height: Option<f32>,
    background: Option<Color>,
    thumb: Color,
    offset: Cell<f32>,
}

impl Scroll {
    pub fn new(child: impl Widget + 'static) -> Self {
        Self {
            child: Box::new(child),
            height: None,
            background: None,
            thumb: Color::rgba(1.0, 1.0, 1.0, 0.35),
            offset: Cell::new(0.0),
        }
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    pub fn background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    fn viewport(&self, bounds: Rect) -> Rect {
        Rect::new(
            bounds.origin.x,
            bounds.origin.y,
            (bounds.size.width - GUTTER).max(0.0),
            bounds.size.height,
        )
    }

    fn content_height(&self, width: f32) -> f32 {
        self.child
            .measure(Size {
                width,
                height: f32::INFINITY,
            })
            .height
    }
}

fn masked(event: &Event, bounds: Rect) -> Event {
    match event {
        Event::PointerPressed(point) if !bounds.contains(*point) => Event::PointerPressed(AWAY),
        Event::PointerMoved(point) if !bounds.contains(*point) => Event::PointerMoved(AWAY),
        Event::PointerReleased(point) if !bounds.contains(*point) => Event::PointerReleased(AWAY),
        other => other.clone(),
    }
}

impl Widget for Scroll {
    fn measure(&self, available: Size) -> Size {
        let natural = self.child.measure(available).width + GUTTER;
        let width = if available.width.is_finite() {
            natural.min(available.width)
        } else {
            natural
        };
        Size {
            width,
            height: self.height.unwrap_or(0.0),
        }
    }

    fn paint(&self, bounds: Rect, painter: &mut dyn Painter) {
        if let Some(color) = self.background {
            painter.fill_rect(bounds, color);
        }

        let viewport = self.viewport(bounds);
        let content = self.content_height(viewport.size.width);
        let max = (content - viewport.size.height).max(0.0);
        let offset = self.offset.get().clamp(0.0, max);
        self.offset.set(offset);

        painter.push_clip(bounds);
        self.child.paint(
            Rect::new(
                viewport.origin.x,
                viewport.origin.y - offset,
                viewport.size.width,
                content,
            ),
            painter,
        );

        if max > 0.0 {
            let track = viewport.size.height;
            let thumb = (track * track / content).clamp(MIN_THUMB.min(track), track);
            let y = bounds.origin.y + (track - thumb) * offset / max;
            painter.fill_rect(
                Rect::new(
                    bounds.origin.x + bounds.size.width - THUMB_WIDTH - THUMB_INSET,
                    y,
                    THUMB_WIDTH,
                    thumb,
                ),
                self.thumb,
            );
        }
        painter.pop_clip();
    }

    fn event(&mut self, bounds: Rect, event: &Event) -> bool {
        let viewport = self.viewport(bounds);
        let content = self.content_height(viewport.size.width);
        let max = (content - viewport.size.height).max(0.0);
        let offset = self.offset.get().clamp(0.0, max);

        if let Event::Scroll(point, delta) = event {
            if !bounds.contains(*point) {
                return false;
            }
            let next = (offset + delta).clamp(0.0, max);
            self.offset.set(next);
            return next != offset;
        }

        self.offset.set(offset);
        let child_bounds = Rect::new(
            viewport.origin.x,
            viewport.origin.y - offset,
            viewport.size.width,
            content,
        );
        self.child.event(child_bounds, &masked(event, bounds))
    }
}
