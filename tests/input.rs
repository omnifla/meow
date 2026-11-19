use std::cell::RefCell;
use std::rc::Rc;

use meow::base::{Event, Key, Modifiers, Point, Rect, Size, Widget};
use meow::clipboard;
use meow::reactive::Signal;
use meow::style::Color;
use meow::widgets::{InputStyle, TextInput};

const INSIDE: Point = Point { x: 20.0, y: 10.0 };
const OUTSIDE: Point = Point { x: 500.0, y: 500.0 };

fn setup() -> (TextInput, Signal<String>, Rect) {
    let color = Color::rgba(1.0, 1.0, 1.0, 1.0);
    let style = InputStyle {
        background: color,
        focused: color,
        text: color,
        selection: color,
    };
    let value = Signal::new(String::new());
    let input = TextInput::new(
        Size {
            width: 200.0,
            height: 30.0,
        },
        style,
        value.clone(),
    );
    (input, value, Rect::new(0.0, 0.0, 200.0, 30.0))
}

fn focus(input: &mut TextInput, bounds: Rect) {
    input.event(bounds, &Event::PointerPressed(INSIDE));
}

fn type_text(input: &mut TextInput, bounds: Rect, text: &str) {
    for ch in text.chars() {
        input.event(bounds, &Event::Char(ch));
    }
}

fn press(input: &mut TextInput, bounds: Rect, key: Key) {
    input.event(bounds, &Event::KeyPressed(key, Modifiers::default()));
}

fn press_shift(input: &mut TextInput, bounds: Rect, key: Key) {
    let modifiers = Modifiers {
        shift: true,
        ..Modifiers::default()
    };
    input.event(bounds, &Event::KeyPressed(key, modifiers));
}

#[test]
fn ignores_typing_until_focused() {
    let (mut input, value, bounds) = setup();
    type_text(&mut input, bounds, "a");
    assert_eq!(value.get(), "");
}

#[test]
fn typing_inserts_at_the_cursor() {
    let (mut input, value, bounds) = setup();
    focus(&mut input, bounds);
    type_text(&mut input, bounds, "hi");
    press(&mut input, bounds, Key::Left);
    type_text(&mut input, bounds, "!");
    assert_eq!(value.get(), "h!i");
}

#[test]
fn backspace_and_delete_remove_around_the_cursor() {
    let (mut input, value, bounds) = setup();
    focus(&mut input, bounds);
    type_text(&mut input, bounds, "abc");
    press(&mut input, bounds, Key::Backspace);
    assert_eq!(value.get(), "ab");
    press(&mut input, bounds, Key::Left);
    press(&mut input, bounds, Key::Delete);
    assert_eq!(value.get(), "a");
}

#[test]
fn clicking_outside_removes_focus() {
    let (mut input, value, bounds) = setup();
    focus(&mut input, bounds);
    input.event(bounds, &Event::PointerPressed(OUTSIDE));
    type_text(&mut input, bounds, "x");
    assert_eq!(value.get(), "");
}

#[test]
fn multibyte_characters_edit_cleanly() {
    let (mut input, value, bounds) = setup();
    focus(&mut input, bounds);
    type_text(&mut input, bounds, "é");
    assert_eq!(value.get(), "é");
    press(&mut input, bounds, Key::Backspace);
    assert_eq!(value.get(), "");
}

#[test]
fn enter_submits_the_current_value() {
    let submitted = Rc::new(RefCell::new(Vec::new()));
    let log = Rc::clone(&submitted);
    let (input, _, bounds) = setup();
    let mut input = input.on_submit(move |text| log.borrow_mut().push(text.to_string()));
    focus(&mut input, bounds);
    type_text(&mut input, bounds, "hey");
    press(&mut input, bounds, Key::Enter);
    assert_eq!(*submitted.borrow(), vec!["hey".to_string()]);
}

#[test]
fn select_all_then_typing_replaces_everything() {
    let (mut input, value, bounds) = setup();
    focus(&mut input, bounds);
    type_text(&mut input, bounds, "abc");
    input.event(bounds, &Event::SelectAll);
    type_text(&mut input, bounds, "x");
    assert_eq!(value.get(), "x");
}

#[test]
fn shift_arrows_select_and_backspace_deletes_the_selection() {
    let (mut input, value, bounds) = setup();
    focus(&mut input, bounds);
    type_text(&mut input, bounds, "abcd");
    press_shift(&mut input, bounds, Key::Left);
    press_shift(&mut input, bounds, Key::Left);
    press(&mut input, bounds, Key::Backspace);
    assert_eq!(value.get(), "ab");
}

#[test]
fn copy_puts_the_selection_on_the_clipboard() {
    let (mut input, value, bounds) = setup();
    focus(&mut input, bounds);
    type_text(&mut input, bounds, "hello");
    input.event(bounds, &Event::SelectAll);
    input.event(bounds, &Event::Copy);
    assert_eq!(clipboard::take_write(), Some("hello".to_string()));
    assert_eq!(value.get(), "hello");
}

#[test]
fn cut_copies_and_removes_the_selection() {
    let (mut input, value, bounds) = setup();
    focus(&mut input, bounds);
    type_text(&mut input, bounds, "hello");
    input.event(bounds, &Event::SelectAll);
    input.event(bounds, &Event::Cut);
    assert_eq!(clipboard::take_write(), Some("hello".to_string()));
    assert_eq!(value.get(), "");
}

#[test]
fn paste_inserts_text_and_strips_newlines() {
    let (mut input, value, bounds) = setup();
    focus(&mut input, bounds);
    input.event(bounds, &Event::Paste("a\nb".to_string()));
    assert_eq!(value.get(), "ab");
}

#[test]
fn paste_replaces_the_selection() {
    let (mut input, value, bounds) = setup();
    focus(&mut input, bounds);
    type_text(&mut input, bounds, "abc");
    input.event(bounds, &Event::SelectAll);
    input.event(bounds, &Event::Paste("z".to_string()));
    assert_eq!(value.get(), "z");
}
