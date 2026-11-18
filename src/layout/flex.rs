use taffy::prelude::*;

use crate::base::{Rect, Size};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Direction {
    Row,
    Column,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Align {
    Start,
    Center,
    End,
    #[default]
    Stretch,
}

#[derive(Clone, Copy, Debug)]
pub struct Item {
    pub size: Size,
    pub padding: f32,
    pub grow: f32,
    pub align: Align,
}

#[derive(Clone, Copy, Debug)]
pub struct Spec {
    pub direction: Direction,
    pub gap: f32,
    pub padding: f32,
}

pub fn solve(spec: Spec, items: &[Item], available: Option<Size>) -> (Size, Vec<Rect>) {
    let mut tree: TaffyTree<()> = TaffyTree::new();
    let row = spec.direction == Direction::Row;

    let children: Vec<NodeId> = items
        .iter()
        .map(|item| {
            let stretch = item.align == Align::Stretch;
            let (width, height) = match (stretch, row) {
                (true, true) => (length(item.size.width), auto()),
                (true, false) => (auto(), length(item.size.height)),
                _ => (length(item.size.width), length(item.size.height)),
            };
            let (min_width, min_height) = match (stretch, row) {
                (true, true) => (auto(), length(item.size.height)),
                (true, false) => (length(item.size.width), auto()),
                _ => (auto(), auto()),
            };
            let align_self = match item.align {
                Align::Start => AlignItems::FlexStart,
                Align::Center => AlignItems::Center,
                Align::End => AlignItems::FlexEnd,
                Align::Stretch => AlignItems::Stretch,
            };

            tree.new_leaf(Style {
                size: taffy::Size { width, height },
                min_size: taffy::Size {
                    width: min_width,
                    height: min_height,
                },
                margin: taffy::Rect {
                    left: length(item.padding),
                    right: length(item.padding),
                    top: length(item.padding),
                    bottom: length(item.padding),
                },
                flex_grow: item.grow,
                flex_shrink: 0.0,
                align_self: Some(align_self),
                ..Default::default()
            })
            .expect("taffy leaf")
        })
        .collect();

    let root_size = match available {
        Some(size) => taffy::Size {
            width: length(size.width),
            height: length(size.height),
        },
        None => taffy::Size {
            width: auto(),
            height: auto(),
        },
    };

    let root = tree
        .new_with_children(
            Style {
                flex_direction: if row {
                    FlexDirection::Row
                } else {
                    FlexDirection::Column
                },
                size: root_size,
                padding: taffy::Rect {
                    left: length(spec.padding),
                    right: length(spec.padding),
                    top: length(spec.padding),
                    bottom: length(spec.padding),
                },
                gap: taffy::Size {
                    width: length(spec.gap),
                    height: length(spec.gap),
                },
                ..Default::default()
            },
            &children,
        )
        .expect("taffy root");

    tree.compute_layout(root, taffy::Size::MAX_CONTENT)
        .expect("taffy layout");

    let root_layout = tree.layout(root).expect("taffy root layout");
    let size = Size {
        width: root_layout.size.width,
        height: root_layout.size.height,
    };
    let rects = children
        .iter()
        .map(|&id| {
            let layout = tree.layout(id).expect("taffy child layout");
            Rect::new(
                layout.location.x,
                layout.location.y,
                layout.size.width,
                layout.size.height,
            )
        })
        .collect();

    (size, rects)
}