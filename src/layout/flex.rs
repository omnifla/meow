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
    pub shrink: f32,
    pub align: Align,
}

#[derive(Clone, Copy, Debug)]
pub struct Spec {
    pub direction: Direction,
    pub gap: f32,
    pub padding: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Fixed {
    pub width: Option<f32>,
    pub height: Option<f32>,
}

const UNBOUNDED: Size = Size {
    width: f32::INFINITY,
    height: f32::INFINITY,
};

fn content_space(available: taffy::style::AvailableSpace) -> f32 {
    match available {
        taffy::style::AvailableSpace::Definite(value) => value,
        taffy::style::AvailableSpace::MinContent => 0.0,
        taffy::style::AvailableSpace::MaxContent => f32::INFINITY,
    }
}

fn offered(limit: f32) -> taffy::style::AvailableSpace {
    if limit.is_finite() {
        taffy::style::AvailableSpace::Definite(limit)
    } else {
        taffy::style::AvailableSpace::MaxContent
    }
}

pub fn solve(spec: Spec, items: &[Item], fixed: Option<Size>) -> (Size, Vec<Rect>) {
    let limit = fixed.unwrap_or(UNBOUNDED);
    let fixed = Fixed {
        width: fixed.map(|size| size.width),
        height: fixed.map(|size| size.height),
    };
    solve_measured(spec, items, fixed, limit, &mut |index, _| items[index].size)
}

pub fn solve_measured(
    spec: Spec,
    items: &[Item],
    fixed: Fixed,
    limit: Size,
    measure: &mut dyn FnMut(usize, Size) -> Size,
) -> (Size, Vec<Rect>) {
    let mut tree: TaffyTree<usize> = TaffyTree::new();
    let row = spec.direction == Direction::Row;

    let children: Vec<NodeId> = items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let align_self = match item.align {
                Align::Start => AlignItems::FlexStart,
                Align::Center => AlignItems::Center,
                Align::End => AlignItems::FlexEnd,
                Align::Stretch => AlignItems::Stretch,
            };

            tree.new_leaf_with_context(
                Style {
                    margin: taffy::Rect {
                        left: length(item.padding),
                        right: length(item.padding),
                        top: length(item.padding),
                        bottom: length(item.padding),
                    },
                    flex_grow: item.grow,
                    flex_shrink: item.shrink,
                    align_self: Some(align_self),
                    ..Default::default()
                },
                index,
            )
            .expect("taffy leaf")
        })
        .collect();

    let root_size = taffy::Size {
        width: match fixed.width {
            Some(width) => length(width),
            None => auto(),
        },
        height: match fixed.height {
            Some(height) => length(height),
            None => auto(),
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

    tree.compute_layout_with_measure(
        root,
        taffy::Size {
            width: offered(limit.width),
            height: offered(limit.height),
        },
        |known, space, _node, context, _style| {
            let Some(index) = context else {
                return taffy::Size {
                    width: 0.0,
                    height: 0.0,
                };
            };
            let room = Size {
                width: known.width.unwrap_or_else(|| content_space(space.width)),
                height: known.height.unwrap_or_else(|| content_space(space.height)),
            };
            let size = measure(*index, room);
            taffy::Size {
                width: known.width.unwrap_or(size.width),
                height: known.height.unwrap_or(size.height),
            }
        },
    )
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
