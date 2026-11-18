use crate::base::{Point, Rect};
use crate::style::Color;

#[cfg(feature = "wgpu")]
mod gpu;

#[cfg(feature = "wgpu")]
pub use gpu::WgpuRenderer;

pub struct GlyphImage {
    pub width: u32,
    pub height: u32,
    pub left: i32,
    pub top: i32,
    pub coverage: Vec<u8>,
}

pub trait Painter {
    fn fill_rect(&mut self, rect: Rect, color: Color);

    fn draw_glyph(
        &mut self,
        _key: u64,
        _origin: Point,
        _color: Color,
        _image: &mut dyn FnMut() -> Option<GlyphImage>,
    ) {
    }
}
