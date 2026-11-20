use super::*;

#[test]
fn editor_scrollbar_tracks_and_drags_long_document() {
    let mut state = test_state();
    state
        .editor
        .replace_text(&(0..100).map(|n| format!("Line {n}\n")).collect::<String>());
    let mut rt = rune::Runtime::new(state, view, (600.0, 760.0), rune::Env::new(false, 1.0));
    rt.frame(0.0);
    rt.frame(0.0);
    let [_, max, height, thumb] = rt.state.editor_scroll.get();
    assert!(max > 0.0 && thumb > 0.0 && thumb < height);
    let actions = rt.presented("actions").unwrap().0;
    assert!(
        actions[1] + actions[3] <= 760.0,
        "actions must remain inside the window"
    );
    let (scroll_x, scroll_y) = rt.find("editor-scroll").unwrap();
    rt.wheel(scroll_x, scroll_y, -100.0);
    rt.frame(0.0);
    assert!(
        rt.state.editor_scroll.get()[0] > 0.0,
        "wheel must scroll the editor"
    );
    let before_drag = rt.state.editor_scroll.get()[0];
    let (x, y) = rt.find("editor-scroll-thumb").unwrap();
    rt.pointer_down(x, y);
    rt.pointer_move(x, y + 80.0);
    rt.pointer_up(x, y + 80.0);
    rt.frame(0.0);
    assert!(rt.state.editor_scroll.get()[0] > before_drag);
}
fn test_state() -> State {
    State {
        status: String::new(),
        ..State::new(
            Config {
                sound: false,
                ..Config::default()
            },
            false,
        )
    }
}
#[test]
fn writing_shortcuts_reach_actions_without_hijacking_settings() {
    use rune::{Key, KeyEvent, Mods};
    let mut state = test_state();
    state.editor.insert("Sample");
    let mut rt = rune::Runtime::new(state, view, (600.0, 760.0), rune::Env::new(false, 1.0));
    rt.on_key = Some(handle_shortcut);
    rt.frame(0.0);
    let (x, y) = rt.find("editor").unwrap();
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    for key in ['r', 'p', '.', 't'] {
        rt.state.work_status.clear();
        rt.key(KeyEvent {
            key: Key::Command(key),
            mods: Mods {
                cmd: true,
                logo: cfg!(target_os = "macos"),
                ..Mods::default()
            },
        });
        rt.frame(0.0);
        assert_eq!(
            rt.state.work_status, "Add an API key in Settings, or use Continue with ChatGPT.",
            "shortcut {key}"
        );
    }
    rt.state.work_status.clear();
    rt.key(KeyEvent {
        key: Key::Command('r'),
        mods: Mods {
            cmd: true,
            logo: cfg!(target_os = "macos"),
            shift: true,
            ..Mods::default()
        },
    });
    assert!(rt.state.work_status.is_empty());
    rt.state.settings_open = true;
    rt.frame(0.0);
    rt.key(KeyEvent {
        key: Key::Command('p'),
        mods: Mods {
            cmd: true,
            logo: cfg!(target_os = "macos"),
            ..Mods::default()
        },
    });
    assert!(rt.state.work_status.is_empty());
}
#[test]
fn startup_clipboard_only_fills_an_empty_editor() {
    let mut editor = TextEdit::default();
    assert!(!paste_startup_clipboard(&mut editor, None));
    assert!(!paste_startup_clipboard(&mut editor, Some("")));
    assert!(paste_startup_clipboard(&mut editor, Some("Café\ntext")));
    assert_eq!(editor.text, "Café\ntext");
    assert!(!paste_startup_clipboard(&mut editor, Some("replacement")));
    assert_eq!(editor.text, "Café\ntext");
}
#[test]
fn keyboard_selection_survives_clicking_an_action() {
    use rune::{Key, KeyEvent, Mods};
    let mut state = test_state();
    state.editor.replace_text("Primero café.\nSegundo café.");
    let mut rt = rune::Runtime::new(state, view, (600.0, 760.0), rune::Env::new(false, 1.0));
    rt.frame(0.0);
    let (x, y) = rt.find("editor").unwrap();
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.frame(0.0);
    rt.key(KeyEvent {
        key: Key::Command('a'),
        mods: Mods {
            cmd: true,
            ..Mods::default()
        },
    });
    rt.frame(0.0);
    assert_eq!(rt.state.editor.selected(), "Primero café.\nSegundo café.");
    rt.key(KeyEvent {
        key: Key::Right,
        mods: Mods::default(),
    });
    for _ in 0..5 {
        rt.key(KeyEvent {
            key: Key::Left,
            mods: Mods {
                shift: true,
                ..Mods::default()
            },
        });
    }
    rt.frame(0.0);
    assert_eq!(rt.state.editor.selected(), "café.");
    let (x, y) = rt.find("polish").unwrap();
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.frame(0.0);
    assert_eq!(rt.state.editor.selected(), "café.");
    assert!(
        rt.state.editor.focused,
        "the selection remains visible when an action is clicked"
    );
    let target = editing_target(&rt.state.editor, Action::Polish).unwrap();
    let payload: serde_json::Value = serde_json::from_str(&target.input()).unwrap();
    assert_eq!(payload["document"], "Primero café.\nSegundo café.");
    assert_eq!(payload["selection"], "café.");
    assert_eq!(
        payload["selection_start_utf8_byte"],
        "Primero café.\nSegundo ".len()
    );
}
#[test]
fn mouse_selection_stays_selected_when_action_is_pressed() {
    let mut state = test_state();
    state.editor.replace_text("First sample sentence.");
    let mut rt = rune::Runtime::new(state, view, (600.0, 760.0), rune::Env::new(false, 1.0));
    rt.frame(0.0);
    let bounds = rt.presented("editor").unwrap().0;
    let start = (bounds[0] + 9.0, bounds[1] + 8.0);
    rt.pointer_down(start.0, start.1);
    rt.pointer_move(start.0 + 45.0, start.1);
    rt.pointer_up(start.0 + 45.0, start.1);
    rt.frame(0.0);
    let selected = rt.state.editor.selected().to_string();
    assert!(!selected.is_empty(), "drag should select a passage");
    assert_ne!(
        selected, rt.state.editor.text,
        "drag selects only part of the document"
    );
    let (x, y) = rt.find("rework").unwrap();
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.frame(0.0);
    assert_eq!(rt.state.editor.selected(), selected);
    assert!(rt.state.editor.focused);
    let target = editing_target(&rt.state.editor, Action::Rework).unwrap();
    assert_eq!(
        &target.document()[target.range().start..target.range().end],
        selected
    );
}
#[test]
fn trash_button_clears_and_undo_restores_document() {
    let mut state = test_state();
    state.editor.replace_text("Café\ntext");
    let mut rt = rune::Runtime::new(state, view, (600.0, 760.0), rune::Env::new(false, 1.0));
    rt.frame(0.0);
    let (x, y) = rt.find("trash").unwrap();
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.frame(0.0);
    assert_eq!(rt.state.editor.text, "");
    rt.state.editor.undo();
    assert_eq!(rt.state.editor.text, "Café\ntext");
}
#[test]
fn undo_and_redo_buttons_preserve_selection_edit_history() {
    let mut state = test_state();
    state.editor.replace_text("café, café, fin");
    let start = "café, ".len();
    state.editor.set_selection(start, start + "café".len());
    let target = editing_target(&state.editor, Action::Polish).unwrap();
    let mut rt = rune::Runtime::new(state, view, (600.0, 760.0), rune::Env::new(false, 1.0));
    rt.frame(0.0);
    rt.update(|s, _| {
        assert!(apply_result(
            s,
            WorkResult {
                target,
                text: "té".into()
            }
        ));
    });
    rt.frame(0.0);
    assert_eq!(rt.state.editor.text, "café, té, fin");
    for (button, expected) in [("undo", "café, café, fin"), ("redo", "café, té, fin")] {
        let (x, y) = rt.find(button).unwrap();
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
        rt.frame(0.0);
        assert_eq!(rt.state.editor.text, expected);
    }
}
#[test]
fn return_in_editor_inserts_a_paragraph_break() {
    use rune::{Key, KeyEvent, Mods};
    let mut state = test_state();
    state.editor.replace_text("First");
    state.editor.set_caret(5);
    let mut rt = rune::Runtime::new(state, view, (600.0, 760.0), rune::Env::new(false, 1.0));
    rt.frame(0.0);
    let (x, y) = rt.find("editor").unwrap();
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.frame(0.0);
    rt.update(|s, _| s.editor.set_caret(5));
    rt.frame(0.0);
    rt.key(KeyEvent {
        key: Key::Enter,
        mods: Mods::default(),
    });
    rt.frame(0.0);
    assert_eq!(rt.state.editor.text, "First\n");
}
#[test]
fn selection_is_sent_with_its_full_document_and_exact_utf8_range() {
    let mut edit = TextEdit::new("Antes. ¡Hola mundo! Después.");
    let start = edit.text.find("¡Hola").unwrap();
    let end = start + "¡Hola mundo!".len();
    edit.set_selection(start, end);
    let target = editing_target(&edit, Action::Rework).unwrap();
    let value: serde_json::Value = serde_json::from_str(&target.input()).unwrap();
    assert_eq!(value["document"], "Antes. ¡Hola mundo! Después.");
    assert_eq!(value["selection"], "¡Hola mundo!");
    assert_eq!(value["selection_start_utf8_byte"], start);
    assert_eq!(value["selection_end_utf8_byte"], end);
    edit.set_caret(0);
    let whole = editing_target(&edit, Action::Polish).unwrap();
    assert_eq!(
        &whole.document()[whole.range().start..whole.range().end],
        edit.text
    );
}
#[test]
fn applying_replaces_selection_and_undo_restores_original() {
    let mut state = test_state();
    state.editor.replace_text("One bad part. The rest.");
    state.editor.set_selection(4, 12);
    let target = editing_target(&state.editor, Action::Rework).unwrap();
    assert!(apply_result(
        &mut state,
        WorkResult {
            target,
            text: "good part".into()
        }
    ));
    assert_eq!(state.editor.text, "One good part. The rest.");
    let next = editing_target(&state.editor, Action::Rework).unwrap();
    assert_eq!(
        &next.document()[next.range().start..next.range().end],
        "good part"
    );
    assert_eq!(next.document(), "One good part. The rest.");
    state.editor.undo();
    assert_eq!(state.editor.text, "One bad part. The rest.");
}
#[test]
fn stale_results_do_not_overwrite_new_edits() {
    let mut state = test_state();
    state.editor.replace_text("Original");
    let target = editing_target(&state.editor, Action::Polish).unwrap();
    state.editor.insert("Changed");
    let current = state.editor.text.clone();
    assert!(!apply_result(
        &mut state,
        WorkResult {
            target,
            text: "Output".into()
        }
    ));
    assert_eq!(state.editor.text, current);
}
#[test]
fn continue_adds_one_sentence_after_selection() {
    let mut state = test_state();
    state.editor.replace_text("First sentence. Next paragraph.");
    state.editor.set_selection(0, "First sentence.".len());
    let target = editing_target(&state.editor, Action::Ideas).unwrap();
    assert!(apply_result(
        &mut state,
        WorkResult {
            target,
            text: "Added sentence.".into()
        }
    ));
    assert_eq!(
        state.editor.text,
        "First sentence. Added sentence. Next paragraph."
    );
    let next = editing_target(&state.editor, Action::Ideas).unwrap();
    assert_eq!(
        &next.document()[next.range().start..next.range().end],
        "First sentence."
    );
    state.editor.undo();
    assert_eq!(state.editor.text, "First sentence. Next paragraph.");
    state.editor.redo();
    assert_eq!(
        state.editor.text,
        "First sentence. Added sentence. Next paragraph."
    );
}
#[test]
fn processing_moves_only_target_glyphs_and_respects_reduced_motion() {
    fn state() -> State {
        let mut s = test_state();
        s.editor.replace_text("Before selected after");
        s.editor.set_selection(7, 15);
        s.processing_target = editing_target(&s.editor, Action::Polish);
        s.working = true;
        s
    }
    fn glyphs(rt: &rune::Runtime<State>) -> Vec<[f32; 4]> {
        let rect = rt.presented("editor").unwrap().0;
        rt.instances
            .iter()
            .filter(|i| {
                i.params[1] == rune::render::KIND_GLYPH
                    && i.rect[0] >= rect[0]
                    && i.rect[0] < rect[0] + rect[2]
                    && i.rect[1] >= rect[1]
                    && i.rect[1] < rect[1] + rect[3]
            })
            .map(|i| i.rect)
            .collect()
    }
    let mut rt = rune::Runtime::new(state(), view, (600.0, 760.0), rune::Env::new(false, 1.0));
    rt.frame(0.0);
    let first = glyphs(&rt);
    rt.frame(0.2);
    let second = glyphs(&rt);
    assert_eq!(first.len(), second.len());
    assert!(first.len() >= 19);
    assert!(
        first
            .iter()
            .zip(&second)
            .any(|(a, b)| (a[1] - b[1]).abs() > 0.2)
    );
    assert!(
        (first[0][1] - second[0][1]).abs() < 0.05,
        "unselected text stays still"
    );
    assert!(
        (first.last().unwrap()[1] - second.last().unwrap()[1]).abs() < 0.05,
        "text after the selection stays still"
    );
    rt.set_system_reduced_motion(true);
    rt.frame(0.0);
    let still = glyphs(&rt);
    rt.frame(0.2);
    assert_eq!(still, glyphs(&rt), "reduced motion keeps letters still");
}
#[test]
fn fullscreen_content_stays_at_a_readable_width() {
    let mut rt = rune::Runtime::new(
        test_state(),
        view,
        (1600.0, 900.0),
        rune::Env::new(false, 1.0),
    );
    rt.frame(0.0);
    let editor = rt.presented("editor-card").unwrap().0;
    assert!(
        editor[2] <= 760.0,
        "editor should stay narrow in fullscreen"
    );
    assert!(editor[0] > 32.0, "editor should be centered");
}
#[test]
fn sound_switch_toggles_and_fits_the_settings_card() {
    let mut state = test_state();
    state.settings_open = true;
    let mut rt = rune::Runtime::new(state, view, (600.0, 760.0), rune::Env::new(false, 1.0));
    rt.frame(0.0);
    let bounds = rt.presented("sound-toggle").unwrap().0;
    assert!(bounds[2] < 150.0, "the switch should be compact");
    let (x, y) = rt.find("sound-toggle").unwrap();
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.frame(0.0);
    assert!(rt.state.sound);
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.frame(0.0);
    assert!(!rt.state.sound);
    let card = rt.presented("settings-card").unwrap().0;
    for key in [
        "sound-toggle",
        "paste-on-startup-toggle",
        "menu-bar-icon-toggle",
    ] {
        let rect = rt.presented(key).unwrap().0;
        assert!(
            rect[0] >= card[0] && rect[0] + rect[2] <= card[0] + card[2],
            "{key} must fit inside the settings card"
        );
    }
    let before = rt.state.menu_bar_icon;
    let (x, y) = rt.find("menu-bar-icon-toggle").unwrap();
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.frame(0.0);
    assert_eq!(rt.state.menu_bar_icon, !before);
}
#[test]
fn tabs_remain_in_place_when_switching_pages() {
    let mut rt = rune::Runtime::new(
        test_state(),
        view,
        (600.0, 760.0),
        rune::Env::new(false, 1.0),
    );
    rt.frame(0.0);
    let before = rt.presented("tabs").unwrap().0;
    rt.state.settings_open = true;
    rt.invalidate();
    rt.frame(0.0);
    let after = rt.presented("tabs").unwrap().0;
    assert_eq!(
        before, after,
        "tabs should not move or stretch between pages"
    );
}
#[test]
fn icon_buttons_have_help_text() {
    let mut rt = rune::Runtime::new(
        test_state(),
        view,
        (600.0, 760.0),
        rune::Env::new(false, 1.0),
    );
    rt.frame(0.0);
    let elements = rt.elements();
    for key in [
        "undo",
        "redo",
        "copy-text",
        "rework",
        "polish",
        "shorten",
        "ideas",
    ] {
        let button = elements
            .iter()
            .find(|element| element.key.as_deref() == Some(key))
            .unwrap();
        assert!(
            button.help.as_ref().is_some_and(|help| !help.is_empty()),
            "{key} needs hover help"
        );
    }
}
#[test]
fn logo_wave_moves_and_respects_reduced_motion() {
    let state = || State {
        settings_open: true,
        ..test_state()
    };
    let mut rt = rune::Runtime::new(state(), view, (600.0, 760.0), rune::Env::new(false, 1.0));
    rt.frame(0.0);
    let start = rt.presented("logo-wave").unwrap().0[0];
    rt.frame(1.0);
    let later = rt.presented("logo-wave").unwrap().0[0];
    assert!(
        later < start,
        "the wave should move across its clipped icon"
    );
    let (x, y) = rt.find("logo").unwrap();
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.frame(0.0);
    assert_eq!(
        rt.presented("logo-slosh-front-1").unwrap().1,
        1.0,
        "restarting the splash must not fade the white water layer"
    );
    assert_eq!(rt.presented("logo-wave").unwrap().1, 1.0);
    let rest = rt.presented_pose("logo-slosh-front-1").unwrap().scale();
    rt.frame(0.1);
    assert_eq!(
        rt.presented("logo-wave").unwrap().1,
        1.0,
        "the upper half must stay opaque throughout the splash"
    );
    assert_ne!(
        rt.presented_pose("logo-slosh-front-1").unwrap().scale(),
        rest
    );
    rt.frame(1.4);
    assert_eq!(
        rt.presented_pose("logo-slosh-front-1").unwrap().scale(),
        rest
    );
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.frame(0.0);
    assert!(rt.find("logo-slosh-front-2").is_some());
    rt.set_system_reduced_motion(true);
    rt.frame(0.0);
    let calm = rt.presented_pose("logo-slosh-front-2").unwrap();
    rt.frame(0.1);
    assert_eq!(rt.presented_pose("logo-slosh-front-2").unwrap(), calm);
    let still = rt.presented("logo-wave").unwrap().0[0];
    rt.frame(1.0);
    assert_eq!(rt.presented("logo-wave").unwrap().0[0], still);
}
#[test]
fn model_choices_update_the_editable_model_id() {
    let state = State {
        settings_open: true,
        ..test_state()
    };
    let mut runtime = rune::Runtime::new(state, view, (600.0, 760.0), rune::Env::new(false, 1.0));
    runtime.frame(0.0);
    let (x, y) = runtime
        .find("gpt-6.1-terra")
        .expect("model choice should be visible");
    runtime.pointer_down(x, y);
    runtime.pointer_up(x, y);
    runtime.frame(0.0);
    assert_eq!(runtime.state.model.text, "gpt-6.1-terra");
}
#[test]
fn chatgpt_sign_in_button_is_offered_for_api_key_users() {
    let state = State {
        settings_open: true,
        ..test_state()
    };
    let mut runtime = rune::Runtime::new(state, view, (600.0, 760.0), rune::Env::new(false, 1.0));
    runtime.frame(0.0);
    let (x, y) = runtime.find("chatgpt-signin").unwrap();
    // Pressing must not attempt a real sign-in from tests: the button is
    // present, labelled, and reachable without opening a browser here.
    let elements = runtime.elements();
    let button = elements
        .iter()
        .find(|element| element.key.as_deref() == Some("chatgpt-signin"))
        .unwrap();
    assert_eq!(button.label.as_deref(), Some("Continue with ChatGPT"));
    assert!(x > 0.0 && y > 0.0);
    assert!(
        runtime.find("chatgpt-signout").is_none(),
        "sign-out must be hidden before signing in"
    );
    assert!(
        runtime.find("model-select").is_none(),
        "the plan select stays hidden for API-key users"
    );
}
#[test]
fn chatgpt_account_shows_sign_out_and_selects_catalog_models() {
    let mut state = State {
        settings_open: true,
        auth: "chatgpt".into(),
        plan_email: Some("user@example.com".into()),
        plan_usage: true,
        catalog: vec![
            ("gpt-plan-a".to_string(), "Plan A".to_string()),
            ("gpt-plan-b".to_string(), "Plan B".to_string()),
        ],
        ..test_state()
    };
    // The saved model is not on the plan catalog, so it switches to the first.
    state.model = TextEdit::new("gpt-plan-a");
    let mut runtime = rune::Runtime::new(state, view, (600.0, 760.0), rune::Env::new(false, 1.0));
    runtime.frame(0.0);
    runtime
        .find("chatgpt-signout")
        .expect("sign out is visible");
    assert!(
        runtime.find("chatgpt-signin").is_none(),
        "sign-in hidden once signed in"
    );
    // The preset cards are an API-key view; a signed-in plan uses the select.
    assert!(
        runtime.find("gpt-6-luna").is_none(),
        "cards hidden under the plan"
    );
    assert!(
        runtime.find("gpt-6-astra").is_none(),
        "cards hidden under the plan"
    );
    let (x, y) = runtime
        .find("model-select")
        .expect("the model select is visible");
    runtime.pointer_down(x, y);
    runtime.pointer_up(x, y);
    runtime.frame(0.0);
    assert!(
        runtime.popup_menu().is_some(),
        "the select opens its catalog menu"
    );
    runtime.resolve_popup(Some("gpt-plan-b")).unwrap();
    runtime.frame(0.0);
    assert_eq!(runtime.state.model.text, "gpt-plan-b");
    // Dismissing the menu leaves the model untouched.
    runtime.pointer_down(x, y);
    runtime.pointer_up(x, y);
    runtime.frame(0.0);
    runtime.resolve_popup(None).unwrap();
    runtime.frame(0.0);
    assert_eq!(runtime.state.model.text, "gpt-plan-b");
}

#[test]
fn apply_catalog_switches_off_plan_models_and_keeps_available_ones() {
    let mut state = test_state();
    state.auth = "chatgpt".into();
    state.model = TextEdit::new("gpt-6-luna");
    apply_catalog(&mut state, vec![("gpt-plan-a".into(), "Plan A".into())]);
    assert_eq!(state.model.text, "gpt-plan-a");
    assert!(state.status.contains("Save settings to apply"));
    // A model that is on the plan stays untouched.
    apply_catalog(
        &mut state,
        vec![
            ("gpt-plan-a".into(), "Plan A".into()),
            ("gpt-plan-b".into(), "Plan B".into()),
        ],
    );
    assert_eq!(state.model.text, "gpt-plan-a");
    assert!(!state.status.contains("switched to Plan B"));
    // The catalog never overrides API-key users.
    let mut key_state = test_state();
    key_state.auth = "api-key".into();
    key_state.model = TextEdit::new("gpt-6-luna");
    apply_catalog(&mut key_state, vec![("gpt-plan-a".into(), "Plan A".into())]);
    assert_eq!(key_state.model.text, "gpt-6-luna");
}
