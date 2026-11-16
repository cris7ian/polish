use super::*;
use rune::LoopSpec;

pub(super) fn write_view(s: &State) -> El {
    col()
        .key("write-page")
        .w_full()
        .grow(1.0)
        .min_h(0.0)
        .gap(16.0)
        .transition(Transition::offset(0.0, -10.0).combined(Transition::blur(1.0)))
        .child(
            col()
                .key("editor-card")
                .w_full()
                .grow(1.0)
                .min_h(120.0)
                .padding(10.0)
                .gap(3.0)
                .radius(16.0)
                .gradient(color(0xfffdfb), color(0xfffaf8))
                .border(1.0, color(0xe9ded8))
                .shadow(2.0, 8.0, color(0x70443e).alpha(0.07))
                .child(
                    row()
                        .w_full()
                        .justify_end()
                        .gap(2.0)
                        .child(
                            el().key("trash")
                                .role("button")
                                .label("Clear document")
                                .keeps_text_selection()
                                .help("Clear the document (undoable)")
                                .focusable(true)
                                .size(34.0, 34.0)
                                .center()
                                .radius(8.0)
                                .hover(color(0x783345).alpha(0.06))
                                .press_scale(0.94)
                                .on_press(|s: &mut State| {
                                    if !s.editor.text.is_empty() {
                                        s.editor.set_selection(0, s.editor.text.len());
                                        s.editor.insert("");
                                        s.work_status.clear();
                                    }
                                })
                                .child(glyph("trash", color(0x9a7f7a), &s.symbols)),
                        )
                        .child(
                            el().key("undo")
                                .role("button")
                                .label("Undo")
                                .keeps_text_selection()
                                .help("Undo the last edit (⌘Z)")
                                .focusable(true)
                                .size(34.0, 34.0)
                                .center()
                                .radius(9.0)
                                .hover(color(0x783345).alpha(0.06))
                                .press_scale(0.94)
                                .on_press(|s: &mut State| {
                                    s.editor.undo();
                                    s.work_status.clear();
                                })
                                .child(glyph("undo", color(0x9a7f7a), &s.symbols)),
                        )
                        .child(
                            el().key("redo")
                                .role("button")
                                .label("Redo")
                                .keeps_text_selection()
                                .help("Redo the last undone edit (⌘⇧Z)")
                                .focusable(true)
                                .size(34.0, 34.0)
                                .center()
                                .radius(9.0)
                                .hover(color(0x783345).alpha(0.06))
                                .press_scale(0.94)
                                .on_press(|s: &mut State| {
                                    s.editor.redo();
                                    s.work_status.clear();
                                })
                                .child(glyph("redo", color(0x9a7f7a), &s.symbols)),
                        )
                        .child(
                            el().key("copy-text")
                                .role("button")
                                .label("Copy text")
                                .help("Copy text")
                                .focusable(!s.editor.text.is_empty())
                                .size(34.0, 34.0)
                                .center()
                                .radius(9.0)
                                .hover(color(0x783345).alpha(0.06))
                                .press_scale(0.94)
                                .on_press_tx(|s: &mut State, tx| {
                                    if !s.editor.text.is_empty() {
                                        tx.copy(s.editor.text.clone());
                                        s.work_status = "Copied".into();
                                    }
                                })
                                .child(glyph("copy", color(0x9a7f7a), &s.symbols)),
                        ),
                )
                .child({
                    let input = text_input(&s.editor, TextStyle::new(15.0, 450.0))
                        .key("editor")
                        .w_full()
                        .min_h(80.0)
                        .px(8.0)
                        .pb(8.0)
                        .color(color(0x38282a))
                        .caret(color(0x38282a), color(0xd57d8a).alpha(0.45))
                        .selection_style(
                            SelectionStyle::new(color(0xd57d8a).alpha(0.45))
                                .inactive(color(0xc9aaa8).alpha(0.45), None),
                        )
                        .processing_text(s.processing_target.as_ref().and_then(|target| {
                            (s.working && s.editor.text == target.document())
                                .then_some((target.range().start, target.range().end))
                        }))
                        .placeholder("Paste or write your text here…", color(0xb5a7a2))
                        .on_edit(|s: &mut State| &mut s.editor)
                        .on_submit(|s: &mut State, _| s.editor.insert("\n"));
                    let input = if s.working
                        && s.processing_target
                            .as_ref()
                            .is_some_and(|target| target.document() == s.editor.text)
                    {
                        input.looping(LoopSpec::slide(1.2, [0.0, 0.0], [0.0, 0.001]))
                    } else {
                        input
                    };
                    let metrics = s.editor_scroll.clone();
                    let thumb = s.editor_thumb.clone();
                    let drag_metrics = s.editor_scroll.clone();
                    row()
                        .w_full()
                        .grow(1.0)
                        .min_h(0.0)
                        .child(
                            col()
                                .key("editor-scroll")
                                .grow(1.0)
                                .min_w(0.0)
                                .min_h(0.0)
                                .scroll(ScrollSpec {
                                    stick_bottom: false,
                                })
                                .on_scroll_offset(move |pos| {
                                    let h = pos.drawn[1];
                                    let max = pos.max[1];
                                    let size = if max > 0.0 && h > 0.0 {
                                        (h * h / (h + max)).clamp(24.0_f32.min(h), h)
                                    } else {
                                        0.0
                                    };
                                    let top = if max > 0.0 {
                                        (h - size) * (pos.offset[1] / max).clamp(0.0, 1.0)
                                    } else {
                                        0.0
                                    };
                                    metrics.set([pos.offset[1], max, h, size]);
                                    thumb.set([3.0, top, 6.0, size]);
                                })
                                .child(input),
                        )
                        .child(
                            el().key("editor-scroll-track").w(12.0).h_full().child(
                                el().key("editor-scroll-thumb")
                                    .absolute(0.0, 0.0)
                                    .live_rect(&s.editor_thumb)
                                    .radius(3.0)
                                    .fill(color(0xc9aeaa))
                                    .a11y_hidden()
                                    .on_drag_any(move |drag, tx| {
                                        let [offset, max, h, size] = drag_metrics.get();
                                        match drag.phase {
                                            rune::Phase::Down => drag.data = offset,
                                            rune::Phase::Move if max > 0.0 && h > size => {
                                                tx.scroll_to_y(
                                                    "editor-scroll",
                                                    drag.data + drag.dy * max / (h - size),
                                                );
                                            }
                                            _ => {}
                                        }
                                    }),
                            ),
                        )
                }),
        )
        .child(
            row()
                .key("actions")
                .w_full()
                .center_y()
                .gap(7.0)
                .child(action_button(
                    "Polish",
                    "polish",
                    Action::Polish,
                    true,
                    s.working,
                    &s.symbols,
                ))
                .child(action_button(
                    "Rework",
                    "rework",
                    Action::Rework,
                    false,
                    s.working,
                    &s.symbols,
                ))
                .child(action_button(
                    "Shorten",
                    "shorten",
                    Action::Shorten,
                    false,
                    s.working,
                    &s.symbols,
                ))
                .child(action_button(
                    "Polish Tweet",
                    "tweet",
                    Action::Tweet,
                    false,
                    s.working,
                    &s.symbols,
                ))
                .child(action_button(
                    "Continue idea",
                    "ideas",
                    Action::Ideas,
                    false,
                    s.working,
                    &s.symbols,
                ))
                .child(el().grow(1.0))
                .child(label(&s.work_status, 11.0, 500.0, 0x9d8b85)),
        )
}
