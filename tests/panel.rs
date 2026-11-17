use meow::base::{Rect, Size, Widget};
use meow::render::Painter;
use meow::style::Color;
use meow::widgets::Panel;

#[derive(Default)]
struct Recorder {
    rects: Vec<(Rect, Color)>,
}

impl Painter for Recorder {
    fn fill_rect(&mut self, rect: Rect, color: Color) {
        self.rects.push((rect, color));
    }
}

#[test]
fn panel_paints_its_bounds() {
    let color = Color::rgba(1.0, 0.0, 0.5, 1.0);
    let panel = Panel {
        size: Size { width: 40.0, height: 20.0 },
        color,
    };
    let bounds = Rect::new(10.0, 10.0, 40.0, 20.0);
    let mut recorder = Recorder::default();

    panel.paint(bounds, &mut recorder);

    assert_eq!(recorder.rects, vec![(bounds, color)]);
}
