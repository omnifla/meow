use crate::base::{Event, Rect, Size, Widget};
use crate::layout::{self, Align, Direction, Fixed, Item, Spec};
use crate::render::Painter;

pub struct Slot {
    widget: Box<dyn Widget>,
    padding: f32,
    grow: f32,
    shrink: f32,
    align: Align,
}

impl Slot {
    pub fn new(widget: impl Widget + 'static) -> Self {
        Self {
            widget: Box::new(widget),
            padding: 0.0,
            grow: 0.0,
            shrink: 0.0,
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

    pub fn shrink(mut self, shrink: f32) -> Self {
        self.shrink = shrink;
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

    fn solve(&self, fixed: Fixed, limit: Size) -> (Size, Vec<Rect>) {
        let items: Vec<Item> = self
            .slots
            .iter()
            .map(|slot| Item {
                size: Size::default(),
                padding: slot.padding,
                grow: slot.grow,
                shrink: slot.shrink,
                align: slot.align,
            })
            .collect();

        let spec = Spec {
            direction: self.direction,
            gap: self.gap,
            padding: self.padding,
        };
        let slots = &self.slots;
        layout::solve_measured(spec, &items, fixed, limit, &mut |index, room| {
            slots[index].widget.measure(room)
        })
    }

    fn solve_in(&self, bounds: Rect) -> (Size, Vec<Rect>) {
        let fixed = Fixed {
            width: Some(bounds.size.width),
            height: Some(bounds.size.height),
        };
        self.solve(fixed, bounds.size)
    }
}

impl Widget for Flex {
    fn measure(&self, available: Size) -> Size {
        let unbounded = Size {
            width: f32::INFINITY,
            height: f32::INFINITY,
        };
        let natural = self.solve(Fixed::default(), unbounded).0;
        if natural.width <= available.width {
            return natural;
        }
        let fixed = Fixed {
            width: Some(available.width),
            height: None,
        };
        self.solve(fixed, available).0
    }

    fn paint(&self, bounds: Rect, painter: &mut dyn Painter) {
        let (_, rects) = self.solve_in(bounds);
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
        let (_, rects) = self.solve_in(bounds);
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
