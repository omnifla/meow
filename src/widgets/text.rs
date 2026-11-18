use std::cell::RefCell;

use cosmic_text::{Attrs, Buffer, Color as TextColor, FontSystem, Metrics, Shaping, SwashCache};

use crate::base::{Rect, Size, Widget};
use crate::render::Painter;
use crate::style::Color;

struct Fonts {
    system: FontSystem,
    cache: SwashCache,
}

thread_local! {
    static FONTS: RefCell<Fonts> = RefCell::new(Fonts {
        system: FontSystem::new(),
        cache: SwashCache::new(),
    });
}

fn to_byte(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn shape(buffer: &mut Buffer, system: &mut FontSystem, content: &str) {
    buffer.set_text(system, content, &Attrs::new(), Shaping::Advanced, None);
    buffer.shape_until_scroll(system, true);
}

pub struct Text {
    buffer: RefCell<Buffer>,
    content: RefCell<String>,
    source: Option<Box<dyn Fn() -> String>>,
    color: Color,
    line_height: f32,
}

impl Text {
    pub fn new(content: &str, size: f32, color: Color) -> Self {
        let line_height = (size * 1.25).round();
        let buffer = FONTS.with(|fonts| {
            let mut fonts = fonts.borrow_mut();
            let mut buffer = Buffer::new(&mut fonts.system, Metrics::new(size, line_height));
            buffer.set_size(&mut fonts.system, None, None);
            shape(&mut buffer, &mut fonts.system, content);
            buffer
        });

        Self {
            buffer: RefCell::new(buffer),
            content: RefCell::new(content.to_string()),
            source: None,
            color,
            line_height,
        }
    }

    pub fn bind(size: f32, color: Color, source: impl Fn() -> String + 'static) -> Self {
        let mut text = Self::new(&source(), size, color);
        text.source = Some(Box::new(source));
        text
    }

    fn sync(&self) {
        let Some(source) = &self.source else {
            return;
        };
        let next = source();
        if next == *self.content.borrow() {
            return;
        }
        FONTS.with(|fonts| {
            let mut fonts = fonts.borrow_mut();
            shape(&mut self.buffer.borrow_mut(), &mut fonts.system, &next);
        });
        *self.content.borrow_mut() = next;
    }
}

impl Widget for Text {
    fn measure(&self, _available: Size) -> Size {
        self.sync();
        let buffer = self.buffer.borrow();
        let mut width = 0.0f32;
        let mut lines = 0u32;
        for run in buffer.layout_runs() {
            width = width.max(run.line_w);
            lines += 1;
        }
        Size {
            width,
            height: lines as f32 * self.line_height,
        }
    }

    fn paint(&self, bounds: Rect, painter: &mut dyn Painter) {
        self.sync();
        let base = TextColor::rgba(
            to_byte(self.color.r),
            to_byte(self.color.g),
            to_byte(self.color.b),
            to_byte(self.color.a),
        );

        FONTS.with(|fonts| {
            let Fonts { system, cache } = &mut *fonts.borrow_mut();
            self.buffer
                .borrow()
                .draw(system, cache, base, |x, y, w, h, color| {
                    if color.a() == 0 {
                        return;
                    }
                    painter.fill_rect(
                        Rect::new(
                            bounds.origin.x + x as f32,
                            bounds.origin.y + y as f32,
                            w as f32,
                            h as f32,
                        ),
                        Color::rgba(
                            color.r() as f32 / 255.0,
                            color.g() as f32 / 255.0,
                            color.b() as f32 / 255.0,
                            color.a() as f32 / 255.0,
                        ),
                    );
                });
        });
    }
}
