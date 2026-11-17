use crate::base::Rect;
use crate::style::Color;

#[cfg(feature = "wgpu")]
mod gpu;

#[cfg(feature = "wgpu")]
pub use gpu::WgpuRenderer;

pub trait Painter {
    fn fill_rect(&mut self, rect: Rect, color: Color);
}
