use crate::base::{Event, Rect, Size, Widget};
use crate::layout::{self, Align, Direction, Item, Spec};
use crate::render::Painter;

pub struct Slot {
    widget: Box<dyn Widget>,
    padding: f32,
    grow: f32,
    align: Align,
}

impl Slot {
    pub fn new(widget: impl Widget + 'static) -> Self {
        Self {
            widget: Box::new(widget),
            padding: 0.0,
            grow: 0.0,
            align: Align::default(),
        }
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    pub fn grow(mut self, grow: f32) -> Self {
        self.grow = grow;
        self
    }

    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }
}

pub struct Flex {
    direction: Direction,
    gap: f32,
    padding: f32,
    slots: Vec<Slot>,
}

impl Flex {
    pub fn row() -> Self {
        Self::new(Direction::Row)
    }

    pub fn column() -> Self {
        Self::new(Direction::Column)
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    pub fn child(self, widget: impl Widget + 'static) -> Self {
        self.slot(Slot::new(widget))
    }

    pub fn slot(mut self, slot: Slot) -> Self {
        self.slots.push(slot);
        self
    }

    fn new(direction: Direction) -> Self {
        Self {
            direction,
            gap: 0.0,
            padding: 0.0,
            slots: Vec::new(),
        }
    }

    fn solve(&self, available: Option<Size>) -> (Size, Vec<Rect>) {
        let unbounded = Size {
            width: f32::INFINITY,
            height: f32::INFINITY,
        };
        let items: Vec<Item> = self
            .slots
            .iter()
            .map(|slot| Item {
                size: slot.widget.measure(available.unwrap_or(unbounded)),
                padding: slot.padding,
                grow: slot.grow,
                align: slot.align,
            })
            .collect();

        let spec = Spec {
            direction: self.direction,
            gap: self.gap,
            padding: self.padding,
        };
        layout::solve(spec, &items, available)
    }
}

impl Widget for Flex {
    fn measure(&self, _available: Size) -> Size {
        self.solve(None).0
    }

    fn paint(&self, bounds: Rect, painter: &mut dyn Painter) {
        let (_, rects) = self.solve(Some(bounds.size));
        for (slot, rect) in self.slots.iter().zip(rects) {
            let child = Rect::new(
                bounds.origin.x + rect.origin.x,
                bounds.origin.y + rect.origin.y,
                rect.size.width,
                rect.size.height,
            );
            slot.widget.paint(child, painter);
        }
    }

    fn event(&mut self, bounds: Rect, event: &Event) -> bool {
        let (_, rects) = self.solve(Some(bounds.size));
        let mut changed = false;
        for (slot, rect) in self.slots.iter_mut().zip(rects) {
            let child = Rect::new(
                bounds.origin.x + rect.origin.x,
                bounds.origin.y + rect.origin.y,
                rect.size.width,
                rect.size.height,
            );
            changed |= slot.widget.event(child, event);
        }
        changed
    }
}
