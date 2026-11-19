use meow::base::Size;
use meow::reactive::Signal;
use meow::style::Color;
use meow::widgets::{Button, ButtonStyle, Flex, InputStyle, Panel, Scroll, Slot, Text, TextInput};

const WHITE: Color = Color::rgba(1.0, 1.0, 1.0, 1.0);

fn build_ui() -> Flex {
    let count = Signal::new(0u32);

    let style = ButtonStyle {
        normal: Color::rgba(0.35, 0.6, 1.0, 1.0),
        hovered: Color::rgba(0.5, 0.72, 1.0, 1.0),
        pressed: Color::rgba(0.2, 0.4, 0.8, 1.0),
    };
    let button = Button::new(
        Size {
            width: 80.0,
            height: 80.0,
        },
        style,
    )
    .label(Text::new("Click", 16.0, WHITE))
    .on_click({
        let count = count.clone();
        move || count.update(|n| *n += 1)
    });

    let counter = Text::bind(24.0, WHITE, move || {
        format!("Clicked {} times", count.get())
    });
    let name = Signal::new(String::new());
    let input = TextInput::new(
        Size {
            width: 240.0,
            height: 36.0,
        },
        InputStyle {
            background: Color::rgba(0.15, 0.15, 0.2, 1.0),
            focused: Color::rgba(0.2, 0.22, 0.32, 1.0),
            selection: Color::rgba(0.3, 0.5, 1.0, 0.5),
            text: WHITE,
        },
        name.clone(),
    )
    .on_submit(|text| println!("submitted: {text}"));
    let greeting = Text::bind(18.0, WHITE, move || format!("Hello, {}!", name.get()));
    let items = (1..=30).fold(Flex::column().gap(6.0), |column, n| {
        column.child(Text::new(&format!("Item {n}"), 16.0, WHITE))
    });
    Flex::column()
        .padding(16.0)
        .gap(12.0)
        .child(Panel {
            size: Size { width: 240.0, height: 60.0 },
            color: Color::rgba(1.0, 0.45, 0.7, 1.0),
        })
        .child(input).child(greeting)
        .child(counter)
        .child(
            Text::new(
                "meow is a declarative, reactive UI framework for Rust. Resize the window and this paragraph rewraps to fit.",
                16.0,
                WHITE,
            )
            .wrap(),
        )
                .child(
            Flex::row()
                .gap(12.0)
                .child(Panel {
                    size: Size { width: 40.0, height: 40.0 },
                    color: Color::rgba(0.7, 0.5, 1.0, 1.0),
                })
                .slot(
                    Slot::new(
                        Text::new(
                            "This text sits next to a swatch in a row, and it shrinks and wraps to fit whatever room is left.",
                            16.0,
                            WHITE,
                        )
                        .wrap(),
                    )
                    .shrink(1.0),
                ),
        )
        .child(
            Flex::row()
                .gap(12.0)
                .child(button)
                .slot(
                    Slot::new(Panel {
                        size: Size { width: 80.0, height: 80.0 },
                        color: Color::rgba(0.4, 0.9, 0.5, 1.0),
                    })
                    .grow(1.0),
                )
                .child(Panel {
                    size: Size { width: 80.0, height: 80.0 },
                    color: Color::rgba(1.0, 0.85, 0.3, 1.0),
                }),
        )
        .slot(
            Slot::new(Scroll::new(items).background(Color::rgba(0.12, 0.12, 0.16, 1.0)))
                .grow(1.0),
        )
}

fn main() {
    meow::platform::run("meow", build_ui());
}
