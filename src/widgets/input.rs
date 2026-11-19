use std::cell::Cell;
use std::time::{Duration, Instant};

use super::Text;
use crate::base::{Event, Key, Modifiers, Point, Rect, Size, Widget};
use crate::clipboard;
use crate::reactive::{self, Signal};
use crate::render::Painter;
use crate::style::Color;

const FONT_SIZE: f32 = 16.0;
const PADDING: f32 = 8.0;
const CARET_WIDTH: f32 = 2.0;
const BLINK_MS: u128 = 530;

#[derive(Clone, Copy, Debug)]
pub struct InputStyle {
    pub background: Color,
    pub focused: Color,
    pub text: Color,
    pub selection: Color,
}

pub struct TextInput {
    size: Size,
    style: InputStyle,
    value: Signal<String>,
    text: Text,
    cursor: usize,
    anchor: usize,
    scroll: f32,
    focused: bool,
    dragging: bool,
    last_input: Cell<Instant>,
    on_submit: Option<Box<dyn FnMut(&str)>>,
}

fn clamp(text: &str, index: usize) -> usize {
    let mut index = index.min(text.len());
    while !text.is_char_boundary(index) {
        index -= 1;
    }
    index
}

fn prev_boundary(text: &str, cursor: usize) -> usize {
    text[..cursor]
        .char_indices()
        .next_back()
        .map(|(index, _)| index)
        .unwrap_or(0)
}

fn next_boundary(text: &str, cursor: usize) -> usize {
    text[cursor..]
        .chars()
        .next()
        .map_or(cursor, |ch| cursor + ch.len_utf8())
}

impl TextInput {
    pub fn new(size: Size, style: InputStyle, value: Signal<String>) -> Self {
        let source = value.clone();
        let text = Text::bind(FONT_SIZE, style.text, move || source.get());
        let end = value.with(|text| text.len());
        Self {
            size,
            style,
            value,
            text,
            cursor: end,
            anchor: end,
            scroll: 0.0,
            focused: false,
            dragging: false,
            last_input: Cell::new(Instant::now()),
            on_submit: None,
        }
    }

    pub fn on_submit(mut self, handler: impl FnMut(&str) + 'static) -> Self {
        self.on_submit = Some(Box::new(handler));
        self
    }

    fn cursor(&self) -> usize {
        self.value.with(|text| clamp(text, self.cursor))
    }

    fn anchor(&self) -> usize {
        self.value.with(|text| clamp(text, self.anchor))
    }

    fn selection(&self) -> Option<(usize, usize)> {
        let (cursor, anchor) = (self.cursor(), self.anchor());
        (cursor != anchor).then(|| (cursor.min(anchor), cursor.max(anchor)))
    }

    fn selected_text(&self) -> Option<String> {
        let (start, end) = self.selection()?;
        Some(self.value.with(|text| text[start..end].to_string()))
    }

    fn touch(&self) {
        self.last_input.set(Instant::now());
    }

    fn move_to(&mut self, index: usize, extend: bool) -> bool {
        let before = (self.cursor, self.anchor);
        self.cursor = index;
        if !extend {
            self.anchor = index;
        }
        (self.cursor, self.anchor) != before
    }

    fn delete_selection(&mut self) -> bool {
        let Some((start, end)) = self.selection() else {
            return false;
        };
        self.value.update(|text| text.replace_range(start..end, ""));
        self.cursor = start;
        self.anchor = start;
        true
    }

    fn insert_str(&mut self, content: &str) {
        self.delete_selection();
        let cursor = self.cursor();
        self.value.update(|text| text.insert_str(cursor, content));
        self.cursor = cursor + content.len();
        self.anchor = self.cursor;
    }

    fn inner(&self, bounds: Rect) -> Rect {
        Rect::new(
            bounds.origin.x + PADDING,
            bounds.origin.y,
            (bounds.size.width - 2.0 * PADDING).max(0.0),
            bounds.size.height,
        )
    }

    fn text_origin(&self, bounds: Rect) -> Point {
        let height = self.text.measure(bounds.size).height;
        Point {
            x: bounds.origin.x + PADDING - self.scroll,
            y: bounds.origin.y + (bounds.size.height - height) / 2.0,
        }
    }

    fn hit(&self, bounds: Rect, point: Point) -> usize {
        let origin = self.text_origin(bounds);
        self.value
            .with(|text| clamp(text, self.text.hit(point.x - origin.x)))
    }

    fn reveal(&mut self, bounds: Rect) {
        let inner = (bounds.size.width - 2.0 * PADDING - CARET_WIDTH).max(0.0);
        if self.text.measure(bounds.size).width <= inner {
            self.scroll = 0.0;
            return;
        }
        let caret = self.text.caret_x(self.cursor());
        if caret - self.scroll > inner {
            self.scroll = caret - inner;
        }
        if caret < self.scroll {
            self.scroll = caret;
        }
        self.scroll = self.scroll.max(0.0);
    }

    fn press(&mut self, key: Key, modifiers: Modifiers) -> bool {
        let cursor = self.cursor();
        let extend = modifiers.shift;
        match key {
            Key::Left => match self.selection() {
                Some((start, _)) if !extend => self.move_to(start, false),
                _ => {
                    let to = self.value.with(|text| prev_boundary(text, cursor));
                    self.move_to(to, extend)
                }
            },
            Key::Right => match self.selection() {
                Some((_, end)) if !extend => self.move_to(end, false),
                _ => {
                    let to = self.value.with(|text| next_boundary(text, cursor));
                    self.move_to(to, extend)
                }
            },
            Key::Home => self.move_to(0, extend),
            Key::End => {
                let end = self.value.with(|text| text.len());
                self.move_to(end, extend)
            }
            Key::Backspace => {
                if self.delete_selection() {
                    return true;
                }
                if cursor == 0 {
                    return false;
                }
                let start = self.value.with(|text| prev_boundary(text, cursor));
                self.value
                    .update(|text| text.replace_range(start..cursor, ""));
                self.cursor = start;
                self.anchor = start;
                true
            }
            Key::Delete => {
                if self.delete_selection() {
                    return true;
                }
                let end = self.value.with(|text| next_boundary(text, cursor));
                if end == cursor {
                    return false;
                }
                self.value
                    .update(|text| text.replace_range(cursor..end, ""));
                true
            }
            Key::Enter => {
                let text = self.value.get();
                if let Some(handler) = self.on_submit.as_mut() {
                    handler(&text);
                }
                true
            }
            Key::Escape => {
                self.focused = false;
                true
            }
        }
    }
}

impl Widget for TextInput {
    fn measure(&self, _available: Size) -> Size {
        self.size
    }

    fn paint(&self, bounds: Rect, painter: &mut dyn Painter) {
        let background = if self.focused {
            self.style.focused
        } else {
            self.style.background
        };
        painter.fill_rect(bounds, background);

        painter.push_clip(self.inner(bounds));
        let origin = self.text_origin(bounds);
        let size = self.text.measure(bounds.size);

        if self.focused {
            if let Some((start, end)) = self.selection() {
                let x0 = origin.x + self.text.caret_x(start);
                let x1 = origin.x + self.text.caret_x(end);
                painter.fill_rect(
                    Rect::new(x0, origin.y, x1 - x0, size.height),
                    self.style.selection,
                );
            }
        }

        self.text.paint(
            Rect::new(origin.x, origin.y, size.width, size.height),
            painter,
        );

        if self.focused {
            let elapsed = self.last_input.get().elapsed().as_millis();
            if (elapsed / BLINK_MS) % 2 == 0 {
                let x = (origin.x + self.text.caret_x(self.cursor())).round();
                painter.fill_rect(
                    Rect::new(x, origin.y, CARET_WIDTH, size.height),
                    self.style.text,
                );
            }
            let remaining = BLINK_MS - elapsed % BLINK_MS;
            reactive::request_wake(Duration::from_millis(remaining as u64));
        }
        painter.pop_clip();
    }

    fn event(&mut self, bounds: Rect, event: &Event) -> bool {
        let changed = match event {
            Event::PointerPressed(point) => {
                let was_focused = self.focused;
                self.focused = bounds.contains(*point);
                self.dragging = self.focused;
                if !self.focused {
                    return was_focused;
                }
                let index = self.hit(bounds, *point);
                self.move_to(index, false);
                true
            }
            Event::PointerMoved(point) if self.dragging => {
                let index = self.hit(bounds, *point);
                self.move_to(index, true)
            }
            Event::PointerReleased(_) => {
                self.dragging = false;
                return false;
            }
            Event::Char(ch) if self.focused => {
                let mut buffer = [0u8; 4];
                self.insert_str(ch.encode_utf8(&mut buffer));
                true
            }
            Event::Paste(content) if self.focused => {
                let line: String = content.chars().filter(|ch| !ch.is_control()).collect();
                self.insert_str(&line);
                true
            }
            Event::SelectAll if self.focused => {
                self.anchor = 0;
                self.cursor = self.value.with(|text| text.len());
                true
            }
            Event::Copy if self.focused => {
                if let Some(text) = self.selected_text() {
                    clipboard::write(text);
                }
                false
            }
            Event::Cut if self.focused => match self.selected_text() {
                Some(text) => {
                    clipboard::write(text);
                    self.delete_selection()
                }
                None => false,
            },
            Event::KeyPressed(key, modifiers) if self.focused => self.press(*key, *modifiers),
            _ => return false,
        };

        if changed {
            self.touch();
            self.reveal(bounds);
        }
        changed
    }
}
