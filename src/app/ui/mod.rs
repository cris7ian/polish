use super::*;
use rune::{
    Color, Cx, El, LineCap, Path, ScrollSpec, SelectionStyle, Shape, TextStyle, Transition, col,
    el, row, text, text_input, tok,
};
use std::time::Duration;

mod editor;
mod logo;
mod settings;
use editor::write_view;
use logo::logo;
use settings::settings_view;

fn color(hex: u32) -> Color {
    Color {
        r: ((hex >> 16) & 255) as f32 / 255.0,
        g: ((hex >> 8) & 255) as f32 / 255.0,
        b: (hex & 255) as f32 / 255.0,
        a: 1.0,
    }
}
fn label(value: impl Into<String>, size: f32, weight: f32, ink: u32) -> El {
    text(value.into())
        .font(TextStyle::new(size, weight))
        .color(color(ink))
}
fn header(settings: bool, splashes: u64) -> El {
    row()
        .key("header")
        .w_full()
        .center_y()
        .justify_between()
        .child(logo(splashes))
        .child(tabs(settings))
}
fn tabs(settings: bool) -> El {
    fn tab(name: &'static str, key: &'static str, active: bool, setting: bool) -> El {
        el().key(key)
            .role("tab")
            .label(name)
            .checked(active)
            .focusable(true)
            .size(if setting { 88.0 } else { 70.0 }, 34.0)
            .center()
            .radius(10.0)
            .hover(color(0x783345).alpha(0.05))
            .on_press(move |s: &mut State| s.settings_open = setting)
            .child(label(
                name,
                12.0,
                650.0,
                if active { 0x513439 } else { 0x9a8881 },
            ))
    }
    row()
        .key("tabs")
        .gap(4.0)
        .padding(4.0)
        .radius(14.0)
        .fill(color(0xeee4df))
        .child(
            el().key("tab-highlight")
                .absolute(if settings { 78.0 } else { 4.0 }, 4.0)
                .size(if settings { 88.0 } else { 70.0 }, 34.0)
                .radius(10.0)
                .fill(color(0xfffdfb))
                .shadow(2.0, 5.0, color(0x73433d).alpha(0.10))
                .motion(tok::motion::SNAPPY),
        )
        .child(tab("Write", "tab-write", !settings, false))
        .child(tab("Settings", "tab-settings", settings, true))
}
fn glyph(kind: &str, ink: Color, symbols: &HashMap<String, rune::Image>) -> El {
    if let Some(image) = symbols.get(kind) {
        return el().size(18.0, 18.0).pixels(image.pixels()).a11y_hidden();
    }
    let width = 1.6;
    let line = |path: Path| Shape::new().stroke_path(&path, width, LineCap::Round, ink);
    let shape = match kind {
        "trash" => line(
            Path::new()
                .move_to([5.0, 6.0])
                .line_to([19.0, 6.0])
                .move_to([9.0, 6.0])
                .line_to([10.0, 3.5])
                .line_to([14.0, 3.5])
                .line_to([15.0, 6.0])
                .move_to([7.0, 8.0])
                .line_to([8.0, 20.0])
                .line_to([16.0, 20.0])
                .line_to([17.0, 8.0]),
        ),
        "undo" => line(
            Path::new()
                .move_to([8.0, 5.0])
                .line_to([3.0, 9.0])
                .line_to([8.0, 13.0])
                .move_to([4.0, 9.0])
                .line_to([14.0, 9.0])
                .cubic_to([21.0, 9.0], [21.0, 18.0], [14.0, 18.0]),
        ),
        "redo" => line(
            Path::new()
                .move_to([16.0, 5.0])
                .line_to([21.0, 9.0])
                .line_to([16.0, 13.0])
                .move_to([20.0, 9.0])
                .line_to([10.0, 9.0])
                .cubic_to([3.0, 9.0], [3.0, 18.0], [10.0, 18.0]),
        ),
        "copy" => line(
            Path::new()
                .move_to([8.0, 7.0])
                .line_to([19.0, 7.0])
                .line_to([19.0, 20.0])
                .line_to([8.0, 20.0])
                .close()
                .move_to([5.0, 16.0])
                .line_to([4.0, 16.0])
                .line_to([4.0, 4.0])
                .line_to([15.0, 4.0])
                .line_to([15.0, 7.0]),
        ),
        "rework" => line(
            Path::new()
                .move_to([4.0, 9.0])
                .cubic_to([6.0, 3.0], [15.0, 2.0], [19.0, 7.0])
                .line_to([19.0, 3.5])
                .move_to([19.0, 7.0])
                .line_to([15.5, 7.0])
                .move_to([20.0, 15.0])
                .cubic_to([18.0, 21.0], [9.0, 22.0], [5.0, 17.0])
                .line_to([5.0, 20.5])
                .move_to([5.0, 17.0])
                .line_to([8.5, 17.0]),
        ),
        "polish" => line(
            Path::new()
                .move_to([12.0, 3.0])
                .quad_to([13.0, 10.0], [20.0, 11.0])
                .quad_to([13.0, 12.0], [12.0, 19.0])
                .quad_to([11.0, 12.0], [4.0, 11.0])
                .quad_to([11.0, 10.0], [12.0, 3.0])
                .close()
                .move_to([19.0, 16.0])
                .line_to([19.0, 21.0])
                .move_to([16.5, 18.5])
                .line_to([21.5, 18.5]),
        ),
        "tweet" => line(
            Path::new()
                .move_to([3.0, 8.0])
                .cubic_to([8.0, 10.0], [11.0, 10.0], [13.0, 5.0])
                .cubic_to([15.0, 2.0], [19.0, 4.0], [19.0, 7.0])
                .line_to([22.0, 6.0])
                .line_to([19.0, 10.0])
                .cubic_to([18.0, 17.0], [10.0, 21.0], [3.0, 17.0])
                .cubic_to([7.0, 17.0], [9.0, 15.0], [10.0, 14.0])
                .cubic_to([6.0, 13.0], [4.0, 11.0], [3.0, 8.0]),
        ),
        "shorten" => line(
            Path::new()
                .move_to([4.0, 7.0])
                .line_to([20.0, 7.0])
                .move_to([6.0, 12.0])
                .line_to([18.0, 12.0])
                .move_to([9.0, 17.0])
                .line_to([15.0, 17.0]),
        ),
        _ => Shape::new()
            .fill(
                vec![[4.0, 11.0], [6.0, 11.0], [6.0, 13.0], [4.0, 13.0]],
                ink,
            )
            .fill(
                vec![[11.0, 11.0], [13.0, 11.0], [13.0, 13.0], [11.0, 13.0]],
                ink,
            )
            .fill(
                vec![[18.0, 11.0], [20.0, 11.0], [20.0, 13.0], [18.0, 13.0]],
                ink,
            ),
    };
    el().size(18.0, 18.0)
        .shape(shape.view_box(24.0, 24.0))
        .a11y_hidden()
}
fn action_button(
    name: &'static str,
    key: &'static str,
    action: Action,
    primary: bool,
    busy: bool,
    symbols: &HashMap<String, rune::Image>,
) -> El {
    let tip = match action {
        Action::Rework => "Rework for clarity and flow (⌘R)",
        Action::Polish => "Fix spelling, grammar, and punctuation (⌘P)",
        Action::Shorten => "Make the text more concise (⌘.)",
        Action::Tweet => "Fit text into a tweet (⌘T)",
        Action::Ideas => "Add a sentence after the selection",
    };
    el().key(key)
        .role("button")
        .label(name)
        .help(tip)
        .help_above()
        .keeps_text_selection()
        .focusable(!busy)
        .size(38.0, 36.0)
        .center()
        .radius(10.0)
        .gradient(
            color(if primary { 0xd95b6d } else { 0xfffdfb }),
            color(if primary { 0xb93650 } else { 0xf5ede9 }),
        )
        .border(1.0, color(if primary { 0xb93750 } else { 0xe6d8d2 }))
        .shadow(
            2.0,
            6.0,
            color(0x9d3549).alpha(if primary { 0.20 } else { 0.07 }),
        )
        .hover(color(0xb84254).alpha(0.06))
        .press(color(0x783345).alpha(0.08))
        .press_scale(0.94)
        .motion(tok::motion::SNAPPY)
        .on_press(move |s: &mut State| begin_action(s, action))
        .child(glyph(
            key,
            color(if primary { 0xffffff } else { 0x785c5b }),
            symbols,
        ))
}
pub(super) fn view(s: &State, cx: &mut Cx<State>) -> El {
    let page = if s.settings_open {
        settings_view(s, cx)
    } else {
        write_view(s)
    };
    col()
        .key("window")
        .w_full()
        .h_full()
        .padding(32.0)
        .pt(30.0)
        .items_center()
        .gradient(color(0xf9f3ef), color(0xeee3df))
        .child(
            col()
                .w_full()
                .max_w(760.0)
                .grow(1.0)
                .min_h(0.0)
                .gap(16.0)
                .child(header(s.settings_open, s.logo_splashes))
                .child(page),
        )
}
