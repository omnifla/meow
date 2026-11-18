use std::cell::{Cell, RefCell};
use std::hash::{DefaultHasher, Hash, Hasher};

use cosmic_text::{
    Attrs, Buffer, CacheKey, FontSystem, Metrics, Shaping, SwashCache, SwashContent,
};

use crate::base::{Point, Rect, Size, Widget};
use crate::render::{GlyphImage, Painter};
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

fn shape(buffer: &mut Buffer, system: &mut FontSystem, content: &str) {
    buffer.set_text(system, content, &Attrs::new(), Shaping::Advanced, None);
    buffer.shape_until_scroll(system, true);
}

fn glyph_key(key: CacheKey) -> u64 {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    hasher.finish()
}

fn rasterize(system: &mut FontSystem, cache: &mut SwashCache, key: CacheKey) -> Option<GlyphImage> {
    let image = cache.get_image_uncached(system, key)?;
    if !matches!(image.content, SwashContent::Mask) {
        return None;
    }
    Some(GlyphImage {
        width: image.placement.width,
        height: image.placement.height,
        left: image.placement.left,
        top: image.placement.top,
        coverage: image.data,
    })
}

pub struct Text {
    buffer: RefCell<Buffer>,
    content: RefCell<String>,
    source: Option<Box<dyn Fn() -> String>>,
    color: Color,
    line_height: f32,
    wrap: bool,
    wrap_width: Cell<Option<f32>>,
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
            wrap: false,
            wrap_width: Cell::new(None),
        }
    }

    pub fn bind(size: f32, color: Color, source: impl Fn() -> String + 'static) -> Self {
        let mut text = Self::new(&source(), size, color);
        text.source = Some(Box::new(source));
        text
    }

    pub fn wrap(mut self) -> Self {
        self.wrap = true;
        self
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

    fn fit(&self, width: f32) {
        if !self.wrap {
            return;
        }
        let target = width.is_finite().then(|| width.max(1.0));
        if self.wrap_width.get() == target {
            return;
        }
        FONTS.with(|fonts| {
            let mut fonts = fonts.borrow_mut();
            let mut buffer = self.buffer.borrow_mut();
            buffer.set_size(&mut fonts.system, target, None);
            buffer.shape_until_scroll(&mut fonts.system, true);
        });
        self.wrap_width.set(target);
    }
}

impl Widget for Text {
    fn measure(&self, available: Size) -> Size {
        self.sync();
        self.fit(available.width);
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
        self.fit(bounds.size.width);
        FONTS.with(|fonts| {
            let Fonts { system, cache } = &mut *fonts.borrow_mut();
            let buffer = self.buffer.borrow();
            for run in buffer.layout_runs() {
                let line_y = run.line_y.round();
                for glyph in run.glyphs.iter() {
                    let physical = glyph.physical((0.0, 0.0), 1.0);
                    let origin = Point {
                        x: bounds.origin.x + physical.x as f32,
                        y: bounds.origin.y + line_y + physical.y as f32,
                    };
                    painter.draw_glyph(
                        glyph_key(physical.cache_key),
                        origin,
                        self.color,
                        &mut || rasterize(system, cache, physical.cache_key),
                    );
                }
            }
        });
    }
}