use meow::base::{Rect, Size};
use meow::layout::{self, Align, Direction, Fixed, Item, Spec};

fn item(width: f32, height: f32, grow: f32, align: Align) -> Item {
    Item {
        size: Size { width, height },
        padding: 0.0,
        grow,
        shrink: 0.0,
        align,
    }
}

fn spec(direction: Direction, gap: f32, padding: f32) -> Spec {
    Spec {
        direction,
        gap,
        padding,
    }
}

fn space(width: f32, height: f32) -> Option<Size> {
    Some(Size { width, height })
}

#[test]
fn row_grow_fills_remaining_width() {
    let items = [
        item(50.0, 20.0, 0.0, Align::Start),
        item(50.0, 20.0, 1.0, Align::Start),
    ];

    let (_, rects) = layout::solve(spec(Direction::Row, 10.0, 0.0), &items, space(200.0, 100.0));

    assert_eq!(rects[0], Rect::new(0.0, 0.0, 50.0, 20.0));
    assert_eq!(rects[1], Rect::new(60.0, 0.0, 140.0, 20.0));
}

#[test]
fn column_stretch_fills_cross_axis() {
    let items = [item(30.0, 20.0, 0.0, Align::Stretch)];

    let (_, rects) = layout::solve(spec(Direction::Column, 0.0, 0.0), &items, space(200.0, 100.0));

    assert_eq!(rects[0], Rect::new(0.0, 0.0, 200.0, 20.0));
}

#[test]
fn empty_container_is_just_its_padding() {
    let (size, rects) = layout::solve(spec(Direction::Row, 5.0, 8.0), &[], None);

    assert!(rects.is_empty());
    assert_eq!(size, Size { width: 16.0, height: 16.0 });
}

#[test]
fn measure_wraps_children_with_gap_and_padding() {
    let items = [
        item(50.0, 20.0, 0.0, Align::Start),
        item(30.0, 40.0, 0.0, Align::Start),
    ];

    let (size, _) = layout::solve(spec(Direction::Row, 10.0, 5.0), &items, None);

    assert_eq!(size, Size { width: 100.0, height: 50.0 });
}

#[test]
fn center_alignment_centers_on_cross_axis() {
    let items = [item(50.0, 20.0, 0.0, Align::Center)];

    let (_, rects) = layout::solve(spec(Direction::Row, 0.0, 0.0), &items, space(200.0, 100.0));

    assert_eq!(rects[0], Rect::new(0.0, 40.0, 50.0, 20.0));
}

#[test]
fn container_padding_offsets_children() {
    let items = [item(50.0, 20.0, 0.0, Align::Start)];

    let (_, rects) = layout::solve(spec(Direction::Row, 0.0, 10.0), &items, space(200.0, 100.0));

    assert_eq!(rects[0], Rect::new(10.0, 10.0, 50.0, 20.0));
}

#[test]
fn slot_padding_offsets_and_inflates() {
    let mut padded = item(50.0, 20.0, 0.0, Align::Start);
    padded.padding = 4.0;

    let (_, placed) = layout::solve(spec(Direction::Row, 0.0, 0.0), &[padded], space(200.0, 100.0));
    let (size, _) = layout::solve(spec(Direction::Row, 0.0, 0.0), &[padded], None);

    assert_eq!(placed[0], Rect::new(4.0, 4.0, 50.0, 20.0));
    assert_eq!(size, Size { width: 58.0, height: 28.0 });
}

#[test]
fn column_grow_splits_remaining_height_by_weight() {
    let items = [
        item(50.0, 20.0, 1.0, Align::Start),
        item(50.0, 20.0, 3.0, Align::Start),
    ];

    let (_, rects) = layout::solve(spec(Direction::Column, 0.0, 0.0), &items, space(100.0, 100.0));

    assert_eq!(rects[0], Rect::new(0.0, 0.0, 50.0, 35.0));
    assert_eq!(rects[1], Rect::new(0.0, 35.0, 50.0, 65.0));
}

#[test]
fn shrinkable_items_give_up_space_equally() {
    let mut flexible = item(0.0, 0.0, 0.0, Align::Start);
    flexible.shrink = 1.0;
    let size = Size { width: 100.0, height: 50.0 };

    let (_, rects) = layout::solve_measured(
        spec(Direction::Row, 0.0, 0.0),
        &[flexible, flexible],
        Fixed {
            width: Some(100.0),
            height: Some(50.0),
        },
        size,
        &mut |_, room| Size {
            width: room.width.min(80.0),
            height: 20.0,
        },
    );

    assert_eq!(rects[0], Rect::new(0.0, 0.0, 50.0, 20.0));
    assert_eq!(rects[1], Rect::new(50.0, 0.0, 50.0, 20.0));
}