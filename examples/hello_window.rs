use meow::base::Size;
use meow::reactive::Signal;
use meow::style::Color;
use meow::widgets::{Button, ButtonStyle, Flex, Panel, Slot, Text};

const WHITE: Color = Color::rgba(1.0, 1.0, 1.0, 1.0);

fn build_ui() -> Flex {
    let count = Signal::new(0u32);

    let style = ButtonStyle {
        normal: Color::rgba(0.35, 0.6, 1.0, 1.0),
        hovered: Color::rgba(0.5, 0.72, 1.0, 1.0),
        pressed: Color::rgba(0.2, 0.4, 0.8, 1.0),
    };
    let button = Button::new(Size { width: 80.0, height: 80.0 }, style)
        .label(Text::new("Click", 16.0, WHITE))
        .on_click({
            let count = count.clone();
            move || count.update(|n| *n += 1)
        });

    let counter = Text::bind(24.0, WHITE, move || format!("Clicked {} times", count.get()));

    Flex::column()
        .padding(16.0)
        .gap(12.0)
        .child(Panel {
            size: Size { width: 240.0, height: 60.0 },
            color: Color::rgba(1.0, 0.45, 0.7, 1.0),
        })
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
}

fn main() {
    meow::platform::run("meow", build_ui());
}