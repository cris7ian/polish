use super::*;

fn setting_toggle(
    name: &'static str,
    id: &'static str,
    enabled: bool,
    toggle: fn(&mut State),
) -> El {
    row()
        .key(format!("{id}-toggle"))
        .role("switch")
        .label(name)
        .checked(enabled)
        .focusable(true)
        .h(32.0)
        .center_y()
        .gap(10.0)
        .on_press(toggle)
        .child(label(name, 12.0, 600.0, 0x795b59).a11y_hidden())
        .child(
            el().size(38.0, 22.0)
                .radius(11.0)
                .fill(color(if enabled { 0xc84e65 } else { 0xd9cdc8 }))
                .motion(tok::motion::SNAPPY)
                .a11y_hidden()
                .child(
                    el().key(format!("{id}-knob"))
                        .size(18.0, 18.0)
                        .absolute(if enabled { 18.0 } else { 2.0 }, 2.0)
                        .radius(9.0)
                        .fill(color(0xffffff))
                        .shadow(1.0, 3.0, color(0x70443e).alpha(0.22)),
                ),
        )
}

fn model_option(name: &str, detail: &str, id: &str, selected: bool) -> El {
    let surface = if selected { 0xfff5f2 } else { 0xfffdfb };
    let border = if selected { 0xd2737d } else { 0xe6ded9 };
    let pressed_id = id.to_string();
    row()
        .key(id)
        .role("radio")
        .label(name)
        .checked(selected)
        .focusable(true)
        .grow(1.0)
        .h(84.0)
        .px(14.0)
        .center_y()
        .gap(11.0)
        .radius(12.0)
        .gradient(
            color(surface),
            color(if selected { 0xfceae7 } else { 0xf9f5f1 }),
        )
        .border(1.0, color(border))
        .shadow(2.0, 5.0, color(0x6c3a36).alpha(0.08))
        .hover(color(0xb84254).alpha(0.04))
        .press(color(0xb84254).alpha(0.10))
        .press_scale(0.985)
        .motion(tok::motion::SNAPPY)
        .on_press(move |s: &mut State| s.model = TextEdit::new(pressed_id.clone()))
        .child(el().size(16.0, 16.0).radius(8.0).border(
            if selected { 5.0 } else { 1.5 },
            color(if selected { 0xb74255 } else { 0xcbbeba }),
        ))
        .child(
            col()
                .grow(1.0)
                .min_w(0.0)
                .gap(3.0)
                .child(label(name, 13.0, 650.0, 0x38282a))
                .child(label(detail, 11.0, 450.0, 0x887a76).w_full().wrap()),
        )
}

fn model_select(s: &State) -> El {
    let current = s
        .catalog
        .iter()
        .find(|(slug, _)| slug == &s.model.text)
        .map(|(_, display)| display.clone())
        .unwrap_or_else(|| s.model.text.clone());
    let placeholder = if s.plan_checking {
        "Loading your models…"
    } else if s.catalog.is_empty() {
        "Models appear after your plan check"
    } else {
        current.as_str()
    };
    row()
        .key("model-select")
        .role("combobox")
        .label("Model")
        .help("Choose a model available on your plan")
        .focusable(true)
        .w_full()
        .h(45.0)
        .px(14.0)
        .center_y()
        .justify_between()
        .radius(10.0)
        .gradient(color(0xf9f5f2), color(0xfffdfb))
        .border(1.0, color(0xe5dad4))
        .hover(color(0x783345).alpha(0.06))
        .press(color(0x783345).alpha(0.10))
        .press_scale(0.99)
        .on_press_tx(|s: &mut State, tx| {
            // The long catalog lives in a native menu, so it scrolls and
            // keyboard-navigates the way the system does everywhere else.
            let options: Vec<(String, String)> = if s.catalog.is_empty() {
                vec![(s.model.text.clone(), s.model.text.clone())]
            } else {
                s.catalog.clone()
            };
            let menu = PopupMenu::picker(options, &s.model.text);
            tx.popup_menu(menu, "model-select", |s: &mut State, picked, _| {
                if let Some(slug) = picked {
                    s.model = TextEdit::new(slug);
                }
            });
        })
        .child(label(placeholder, 14.0, 500.0, 0x38282a))
        .child(glyph("chevron", color(0x795b59), &s.symbols))
}

fn model_cards(s: &State) -> El {
    col()
        .gap(8.0)
        .child(
            row()
                .w_full()
                .gap(10.0)
                .child(model_option(
                    "GPT-6 Luna",
                    "Fast & efficient",
                    "gpt-6-luna",
                    s.model.text == "gpt-6-luna",
                ))
                .child(model_option(
                    "GPT-6.1 Terra",
                    "Natural writing",
                    "gpt-6.1-terra",
                    s.model.text == "gpt-6.1-terra",
                )),
        )
        .child(
            row()
                .w_full()
                .gap(10.0)
                .child(model_option(
                    "GPT-6.1 Sol",
                    "Complex work",
                    "gpt-6.1-sol",
                    s.model.text == "gpt-6.1-sol",
                ))
                .child(model_option(
                    "GPT-6 Astra",
                    "Most expensive, not for this kind of work but you do you",
                    "gpt-6-astra",
                    s.model.text == "gpt-6-astra",
                )),
        )
        .children(
            (!["gpt-6-luna", "gpt-6.1-terra", "gpt-6.1-sol", "gpt-6-astra"]
                .contains(&s.model.text.as_str()))
            .then(|| {
                label(
                    format!("Saved model: {}", s.model.text),
                    11.0,
                    500.0,
                    0x887a76,
                )
            }),
        )
}

fn continue_with_chatgpt(s: &State) -> El {
    let busy = s.signing_in;
    el().key("chatgpt-signin")
        .role("button")
        .label("Continue with ChatGPT")
        .help("Sign in with your ChatGPT account to use your plan")
        .help_above()
        .focusable(!busy)
        .h(45.0)
        .w_full()
        .center()
        .radius(10.0)
        .gradient(color(0xf7efeb), color(0xfffdfb))
        .border(1.0, color(0xe5dad4))
        .hover(color(0x783345).alpha(0.06))
        .press(color(0x783345).alpha(0.10))
        .press_scale(0.99)
        .on_press(move |s: &mut State| start_chatgpt_sign_in(s))
        .child(label(
            if busy {
                "Opening ChatGPT…"
            } else {
                "Continue with ChatGPT"
            },
            13.0,
            650.0,
            0xb74255,
        ))
}

fn chatgpt_signed_in(s: &State) -> El {
    let account = s
        .plan_email
        .clone()
        .unwrap_or_else(|| "ChatGPT account".into());
    col()
        .key("chatgpt-account")
        .gap(8.0)
        .child(
            row()
                .w_full()
                .center_y()
                .justify_between()
                .child(
                    col()
                        .gap(3.0)
                        .child(label("Signed in with ChatGPT", 13.0, 650.0, 0x38282a))
                        .child(label(account, 11.0, 500.0, 0x887a76)),
                )
                .child(
                    el().key("chatgpt-signout")
                        .role("button")
                        .label("Sign out")
                        .focusable(!s.signing_out)
                        .h(34.0)
                        .px(14.0)
                        .center()
                        .radius(10.0)
                        .hover(color(0x783345).alpha(0.06))
                        .press(color(0x783345).alpha(0.10))
                        .on_press(move |s: &mut State| start_chatgpt_sign_out(s))
                        .child(label(
                            if s.signing_out {
                                "Signing out…"
                            } else {
                                "Sign out"
                            },
                            12.0,
                            550.0,
                            0x9e686d,
                        )),
                ),
        )
        .child(
            label(
                "Requests use your ChatGPT plan allowance. Manage apps and limits at chatgpt.com/settings/usage.",
                11.0,
                450.0,
                0x887a76,
            )
            .w_full()
            .wrap(),
        )
}

pub(super) fn settings_view(s: &State, _: &mut Cx<State>) -> El {
    let field = |edit: &TextEdit| {
        text_input(edit, TextStyle::new(14.0, 500.0))
            .color(color(0x38282a))
            .placeholder(
                if s.key_saved {
                    "••••••••••••••••"
                } else {
                    "Paste your API key"
                },
                color(0xb5a7a2),
            )
            .w_full()
            .h(45.0)
            .px(14.0)
            .py(11.0)
            .radius(10.0)
            .gradient(color(0xf9f5f2), color(0xfffdfb))
            .border(1.0, color(0xe5dad4))
    };
    let (dot, connection) = if s.auth == "chatgpt" {
        if s.signing_in {
            (0xc7a365, "Opening ChatGPT…".to_string())
        } else if s.plan_checking {
            (0xc7a365, "Checking your plan…".to_string())
        } else if let Some(problem) = &s.plan_problem {
            (0xc54a58, problem.clone())
        } else if s.plan_email.is_some() && s.plan_usage {
            (0x4f9e72, "Using your ChatGPT plan".to_string())
        } else if s.plan_email.is_some() {
            (0xc7a365, "Signed in — plan usage off".to_string())
        } else {
            (0xc54a58, "Sign in again".to_string())
        }
    } else {
        let (dot, text) = match s.key_status {
            KeyStatus::Missing => (0xc54a58, "No API key"),
            KeyStatus::Checking => (0xc7a365, "Checking key…"),
            KeyStatus::Valid => (0x4f9e72, "Key verified"),
            KeyStatus::Invalid => (0xc54a58, "Invalid API key"),
            KeyStatus::Unavailable => (0xc7a365, "Could not verify key"),
        };
        (dot, text.to_string())
    };
    col()
        .key("settings-page")
        .w_full()
        .grow(1.0)
        .transition(Transition::offset(0.0, 10.0).combined(Transition::blur(1.0)))
        .child(
            col()
                .key("settings-card")
                .w_full()
                .padding(24.0)
                .gap(17.0)
                .radius(20.0)
                .gradient(color(0xfffdfb), color(0xfff9f6))
                .border(1.0, color(0xffffff))
                .shadow(2.0, 5.0, color(0x70443e).alpha(0.08))
                .shadow(14.0, 32.0, color(0x70443e).alpha(0.13))
                .child(
                    row()
                        .w_full()
                        .center_y()
                        .justify_between()
                        .child(
                            col()
                                .gap(3.0)
                                .child(label("Your connection", 16.0, 700.0, 0x38282a))
                                .child(label(
                                    "Connect OpenAI to polish your words.",
                                    12.0,
                                    450.0,
                                    0x927f79,
                                )),
                        )
                        .child(
                            row()
                                .center_y()
                                .gap(7.0)
                                .child(el().size(8.0, 8.0).radius(4.0).fill(color(dot)))
                                .child(label(connection, 11.0, 550.0, dot).w_full().wrap()),
                        ),
                )
                .child(el().w_full().h(1.0).fill(color(0xefe7e2)))
                .child(
                    col()
                        .gap(8.0)
                        .child(
                            row()
                                .w_full()
                                .child(label("API KEY", 10.0, 700.0, 0x917e79)),
                        )
                        .child(
                            row()
                                .w_full()
                                .gap(8.0)
                                .center_y()
                                .child(
                                    field(&s.key)
                                        .key("key")
                                        .single_line()
                                        .secure()
                                        .grow(1.0)
                                        .on_edit(|s: &mut State| &mut s.key),
                                )
                                .child(if s.key_saved && s.auth != "chatgpt" {
                                    el().key("copy-key")
                                        .role("button")
                                        .label("Copy API key")
                                        .help("Copy API key")
                                        .focusable(true)
                                        .size(45.0, 45.0)
                                        .center()
                                        .radius(10.0)
                                        .fill(color(0xf7efeb))
                                        .border(1.0, color(0xe5dad4))
                                        .hover(color(0x783345).alpha(0.06))
                                        .press_scale(0.96)
                                        .on_press_tx(|s: &mut State, tx| {
                                            match credentials::read() {
                                                Ok(key) => {
                                                    tx.copy_secret(key, Duration::from_secs(30));
                                                    s.status =
                                                        "API key copied for 30 seconds".into();
                                                }
                                                Err(_) => {
                                                    s.status = "Could not copy the saved key".into()
                                                }
                                            }
                                        })
                                        .child(glyph("copy", color(0x795b59), &s.symbols))
                                } else {
                                    el().size(0.0, 0.0)
                                }),
                        ),
                )
                .child(
                    col()
                        .gap(8.0)
                        .child(
                            row()
                                .w_full()
                                .child(label("CHATGPT PLAN", 10.0, 700.0, 0x917e79)),
                        )
                        .child(if s.auth == "chatgpt" {
                            chatgpt_signed_in(s)
                        } else {
                            continue_with_chatgpt(s)
                        }),
                )
                .child(
                    col()
                        .gap(8.0)
                        .child(label("MODEL", 10.0, 700.0, 0x917e79))
                        .child(if s.auth == "chatgpt" {
                            model_select(s)
                        } else {
                            model_cards(s)
                        }),
                )
                .child(
                    row()
                        .w_full()
                        .center_y()
                        .gap(24.0)
                        .child(
                            setting_toggle("Sounds", "sound", s.sound, |s| s.sound = !s.sound)
                                .w(100.0),
                        )
                        .child(setting_toggle(
                            "Paste on startup",
                            "paste-on-startup",
                            s.paste_on_startup,
                            |s| s.paste_on_startup = !s.paste_on_startup,
                        ))
                        .child(setting_toggle(
                            "menu bar icon",
                            "menu-bar-icon",
                            s.menu_bar_icon,
                            |s| s.menu_bar_icon = !s.menu_bar_icon,
                        )),
                )
                .child(
                    row()
                        .w_full()
                        .center_y()
                        .gap(12.0)
                        .child(
                            el().key("save")
                                .role("button")
                                .label("Save settings")
                                .focusable(true)
                                .h(42.0)
                                .px(19.0)
                                .center()
                                .radius(11.0)
                                .gradient(color(0xd95b6d), color(0xb93650))
                                .border(1.0, color(0xb93750))
                                .shadow(3.0, 7.0, color(0xb33a50).alpha(0.24))
                                .hover(color(0xffffff).alpha(0.09))
                                .press(color(0x67132e).alpha(0.13))
                                .press_scale(0.97)
                                .motion(tok::motion::SNAPPY)
                                .on_press(save)
                                .child(label("Save settings", 13.0, 650.0, 0xffffff)),
                        )
                        .child(if s.auth != "chatgpt" {
                            el().key("remove")
                                .role("button")
                                .label("Remove key")
                                .focusable(true)
                                .h(42.0)
                                .px(10.0)
                                .center()
                                .radius(10.0)
                                .hover(color(0x783345).alpha(0.06))
                                .press(color(0x783345).alpha(0.10))
                                .on_press(|s: &mut State| {
                                    s.status = match credentials::remove() {
                                        Ok(()) => {
                                            s.key_saved = false;
                                            s.key_check.cancel();
                                            s.key_status = KeyStatus::Missing;
                                            "API key removed".into()
                                        }
                                        Err(e) => format!("Could not remove key: {e}"),
                                    };
                                })
                                .child(label("Remove key", 12.0, 550.0, 0x9e686d))
                        } else {
                            el().size(0.0, 0.0)
                        }),
                )
                .child(label(&s.status, 11.0, 500.0, 0x8a7972).w_full().wrap()),
        )
}
