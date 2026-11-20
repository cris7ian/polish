use crate::{
    config::{Config, load_config, save_config},
    credentials,
    editing::{Action, Target, WorkResult},
    engine::{KeyStatus, run_action, verify_key},
    siwc,
};
use rune::{AppSpec, Latest, Player, TextEdit};
use std::collections::HashMap;

mod ui;
use ui::view;
#[cfg(test)]
mod tests;

struct State {
    key: TextEdit,
    model: TextEdit,
    status: String,
    key_saved: bool,
    key_status: KeyStatus,
    key_check: Latest<KeyStatus>,
    editor: TextEdit,
    editor_scroll: rune::Live<[f32; 4]>,
    editor_thumb: rune::Live<[f32; 4]>,
    startup_clipboard: Latest<Option<String>>,
    work: Latest<Result<WorkResult, String>>,
    processing_target: Option<Target>,
    work_status: String,
    working: bool,
    settings_open: bool,
    logo_splashes: u64,
    sound: bool,
    paste_on_startup: bool,
    menu_bar_icon: bool,
    ready_sounds: HashMap<Action, Player>,
    error_sound: Option<Player>,
    symbol_cache: Option<rune::ImageCache>,
    symbols: HashMap<String, rune::Image>,
    /// "api-key" or "chatgpt".
    auth: String,
    plan_email: Option<String>,
    plan_usage: bool,
    plan_checking: bool,
    signing_in: bool,
    signing_out: bool,
    plan_problem: Option<String>,
    catalog: Vec<(String, String)>,
    sign_in: Latest<Result<siwc::SignInOutcome, String>>,
    sign_out: Latest<Result<String, String>>,
    plan_check: Latest<Result<siwc::PlanCheck, String>>,
}
impl Default for State {
    fn default() -> Self {
        Self::new(load_config(), credentials::read().is_ok())
    }
}
impl State {
    fn new(cfg: Config, key_saved: bool) -> Self {
        let auth = cfg.auth.clone();
        let connected_with_plan = auth == "chatgpt" && credentials::read_oauth().is_ok();
        let plan_email = if connected_with_plan {
            siwc::signed_in_email()
        } else {
            None
        };
        let plan_usage = connected_with_plan && siwc::signed_in_plan_usage();
        let status = if connected_with_plan {
            "Connected with your ChatGPT plan"
        } else if key_saved {
            "API key saved in Keychain"
        } else {
            "Add an OpenAI API key to get started"
        };
        Self {
            key: TextEdit::default().secure(),
            model: TextEdit::new(&cfg.model),
            status: status.into(),
            key_saved,
            key_status: if key_saved && auth != "chatgpt" {
                KeyStatus::Checking
            } else {
                KeyStatus::Missing
            },
            key_check: Latest::default(),
            editor: TextEdit::default(),
            editor_scroll: rune::Live::new([0.0; 4]),
            editor_thumb: rune::Live::new([0.0; 4]),
            startup_clipboard: Latest::default(),
            work: Latest::default(),
            processing_target: None,
            work_status: String::new(),
            working: false,
            settings_open: false,
            logo_splashes: 0,
            sound: cfg.sound,
            paste_on_startup: cfg.paste_on_startup,
            menu_bar_icon: cfg.menu_bar_icon,
            ready_sounds: HashMap::new(),
            error_sound: None,
            symbol_cache: None,
            symbols: HashMap::new(),
            auth,
            plan_email,
            plan_usage,
            plan_checking: connected_with_plan,
            signing_in: false,
            signing_out: false,
            plan_problem: None,
            catalog: Vec::new(),
            sign_in: Latest::default(),
            sign_out: Latest::default(),
            plan_check: Latest::default(),
        }
    }
}
fn paste_startup_clipboard(editor: &mut TextEdit, text: Option<&str>) -> bool {
    if !editor.text.is_empty() {
        return false;
    }
    if let Some(text) = text.filter(|text| !text.is_empty()) {
        editor.insert(text);
        true
    } else {
        false
    }
}
fn save(s: &mut State) {
    let model = s.model.text.trim();
    if model.is_empty() {
        s.status = "Enter a model ID.".into();
        return;
    }
    let typed_key = s.key.text.trim().to_string();
    let result = (|| {
        if !typed_key.is_empty() {
            credentials::write(&typed_key)?;
        }
        let mut config = load_config();
        config.model = model.into();
        config.sound = s.sound;
        config.paste_on_startup = s.paste_on_startup;
        config.menu_bar_icon = s.menu_bar_icon;
        // Saving an API key switches the active connection to it; saving with
        // the key field empty keeps the current method untouched.
        if !typed_key.is_empty() {
            config.auth = "api-key".into();
        }
        save_config(&config)
    })();
    match result {
        Ok(()) => {
            if !typed_key.is_empty() {
                s.key_saved = true;
                s.auth = "api-key".into();
                s.plan_problem = None;
            }
            s.key = TextEdit::default().secure();
            s.status = "Settings saved".into();
            if s.auth == "chatgpt" {
                s.plan_checking = true;
                s.plan_check.request((), |_| siwc::verify_plan());
            } else if s.key_saved {
                s.key_status = KeyStatus::Checking;
                s.key_check.request((), |_| verify_key());
            }
        }
        Err(e) => s.status = format!("Could not save: {e}"),
    }
}
fn start_chatgpt_sign_in(s: &mut State) {
    if s.signing_in {
        return;
    }
    s.signing_in = true;
    s.plan_problem = None;
    s.status = "Opening ChatGPT in your browser…".into();
    s.sign_in.request((), |_| siwc::sign_in());
}

fn start_chatgpt_sign_out(s: &mut State) {
    if s.signing_out {
        return;
    }
    s.signing_out = true;
    s.status = "Signing out…".into();
    s.sign_out.request((), |_| siwc::sign_out());
}

/// Shows the account's model catalog and follows it when the saved model is
/// not on the plan. Switching models only edits the field; the user saves.
fn apply_catalog(s: &mut State, catalog: Vec<(String, String)>) {
    s.catalog = catalog;
    if s.auth != "chatgpt" || s.catalog.is_empty() {
        return;
    }
    if !s.catalog.iter().any(|(slug, _)| slug == &s.model.text) {
        let (slug, display) = s.catalog[0].clone();
        s.model = TextEdit::new(slug);
        s.status = format!(
            "Your saved model is not on your ChatGPT plan — switched to {display}. Save settings to apply."
        );
    }
}

fn handle_shortcut(s: &mut State, event: &rune::KeyEvent, tx: &mut rune::Tx) -> bool {
    if s.settings_open {
        return false;
    }
    for (chord, action) in [
        ("mod+r", Some(Action::Rework)),
        ("mod+p", Some(Action::Polish)),
        ("mod+.", Some(Action::Shorten)),
        ("mod+t", Some(Action::Tweet)),
        ("mod+z", None),
        ("mod+shift+z", None),
    ] {
        if rune::Chord::parse(chord)
            .expect("valid shortcut")
            .matches_tx(event, tx)
        {
            if !tx.key_repeat() {
                if let Some(action) = action {
                    begin_action(s, action);
                } else if event.mods.shift {
                    s.editor.redo();
                    s.work_status.clear();
                } else {
                    s.editor.undo();
                    s.work_status.clear();
                }
            }
            return true;
        }
    }
    false
}
fn begin_action(s: &mut State, action: Action) {
    if s.working {
        return;
    }
    let Some(target) = editing_target(&s.editor, action) else {
        s.work_status = "Add some text first.".into();
        return;
    };
    if s.auth == "chatgpt" {
        if credentials::read_oauth().is_err() {
            s.work_status = "Open Settings to sign in with ChatGPT again.".into();
            return;
        }
    } else if !s.key_saved {
        s.work_status = "Add an API key in Settings, or use Continue with ChatGPT.".into();
        return;
    }
    s.working = true;
    s.processing_target = Some(target.clone());
    s.work_status = "Working…".into();
    let model = load_config().model;
    s.work
        .request((target, model), |(target, model)| run_action(target, model));
}
fn apply_result(s: &mut State, done: WorkResult) -> bool {
    if s.editor.text != done.target.document() {
        s.work_status = "Text changed. Try again.".into();
        return false;
    }
    if done.target.action() == Action::Ideas {
        let insertion = done.target.range().end;
        let before = &s.editor.text[..insertion];
        let after = &s.editor.text[insertion..];
        let leading = if before.chars().last().is_some_and(char::is_whitespace) {
            ""
        } else {
            " "
        };
        let trailing = if after.is_empty() || after.chars().next().is_some_and(char::is_whitespace)
        {
            ""
        } else {
            " "
        };
        let addition = format!("{leading}{}{trailing}", done.text.trim());
        s.editor.set_selection(insertion, insertion);
        s.editor.insert(&addition);
        if done.target.has_selection() {
            s.editor
                .set_selection(done.target.range().start, done.target.range().end);
        }
    } else {
        s.editor
            .set_selection(done.target.range().start, done.target.range().end);
        s.editor.insert(&done.text);
        if done.target.has_selection() {
            s.editor.set_selection(
                done.target.range().start,
                done.target.range().start + done.text.len(),
            );
        }
    }
    s.work_status = "Ready".into();
    true
}
fn editing_target(editor: &TextEdit, action: Action) -> Option<Target> {
    Target::from_selection(editor.text.clone(), editor.range(), action)
}

pub(crate) fn run() {
    let mut spec = AppSpec::new("Polish", (600.0, 760.0), State::default, view);
    spec.on_key = Some(handle_shortcut);
    spec.setup = Some(|s, waker| {
        s.startup_clipboard.set_waker(waker.clone());
        if s.paste_on_startup {
            s.startup_clipboard.request((), |_| {
                arboard::Clipboard::new()
                    .ok()
                    .and_then(|mut clipboard| clipboard.get_text().ok())
            });
        }
        s.key_check.set_waker(waker.clone());
        s.work.set_waker(waker.clone());
        s.sign_in.set_waker(waker.clone());
        s.sign_out.set_waker(waker.clone());
        s.plan_check.set_waker(waker.clone());
        let cache = rune::ImageCache::new(waker);
        if let Some(resources) = bundle_resources() {
            for name in [
                "trash", "undo", "redo", "copy", "rework", "polish", "shorten", "tweet", "ideas",
            ] {
                let path = resources.join("Symbols").join(format!("{name}.png"));
                if path.is_file() {
                    s.symbols.insert(name.into(), cache.file(path, 24.0, 1.0));
                }
            }
            for action in [
                Action::Polish,
                Action::Rework,
                Action::Shorten,
                Action::Tweet,
                Action::Ideas,
            ] {
                s.ready_sounds.insert(
                    action,
                    Player::audio(
                        resources
                            .join(action.sound_file())
                            .to_string_lossy()
                            .into_owned(),
                    ),
                );
            }
            s.error_sound = Some(Player::audio(
                resources.join("error.wav").to_string_lossy().into_owned(),
            ));
        }
        s.symbol_cache = Some(cache);
        if s.auth == "chatgpt" {
            if credentials::read_oauth().is_ok() {
                s.plan_checking = true;
                s.plan_check.request((), |_| siwc::verify_plan());
            }
        } else if s.key_saved {
            s.key_check.request((), |_| verify_key());
        }
    });
    spec.poll = Some(|s, _, _| {
        let mut changed = false;
        if let Some(text) = s.startup_clipboard.take() {
            changed |= paste_startup_clipboard(&mut s.editor, text.as_deref());
        }
        if let Some(result) = s.key_check.take() {
            s.key_status = result;
            changed = true;
        }
        if let Some(result) = s.sign_in.take() {
            s.signing_in = false;
            match result {
                Ok(outcome) => {
                    s.auth = "chatgpt".into();
                    s.plan_email = Some(outcome.email.clone());
                    s.plan_usage = outcome.plan_usage;
                    s.plan_problem = None;
                    s.plan_checking = false;
                    apply_catalog(s, outcome.catalog);
                    s.status = if outcome.plan_usage {
                        format!("Signed in as {} — using your ChatGPT plan", outcome.email)
                    } else {
                        "Signed in — allow plan usage in your browser to use the subscription"
                            .into()
                    };
                }
                Err(error) => {
                    s.status = error;
                }
            }
            changed = true;
        }
        if let Some(result) = s.sign_out.take() {
            s.signing_out = false;
            match result {
                Ok(message) => {
                    s.auth = "api-key".into();
                    s.plan_email = None;
                    s.plan_usage = false;
                    s.plan_checking = false;
                    s.plan_problem = None;
                    s.catalog.clear();
                    s.status = message;
                }
                Err(error) => s.status = error,
            }
            changed = true;
        }
        if let Some(result) = s.plan_check.take() {
            s.plan_checking = false;
            match result {
                Ok(check) => {
                    s.plan_email = Some(check.email);
                    s.plan_usage = check.plan_usage;
                    s.plan_problem = None;
                    apply_catalog(s, check.catalog);
                }
                Err(error) => {
                    s.plan_problem = Some(error);
                }
            }
            changed = true;
        }
        if let Some(result) = s.work.take() {
            s.working = false;
            s.processing_target = None;
            let completed_action = match result {
                Ok(done) => {
                    let action = done.target.action();
                    apply_result(s, done).then_some(action)
                }
                Err(error) => {
                    s.work_status = error;
                    None
                }
            };
            if s.sound {
                let sound = completed_action.and_then(|action| s.ready_sounds.get(&action));
                // A missing success asset stays silent; errors use the separate failure note.
                let sound = if completed_action.is_some() {
                    sound
                } else {
                    s.error_sound.as_ref()
                };
                if let Some(sound) = sound {
                    sound.seek(0.0);
                    sound.play();
                }
            }
            changed = true;
        }
        changed
    });
    rune::run(spec);
}

fn bundle_resources() -> Option<std::path::PathBuf> {
    let executable = std::env::current_exe().ok()?;
    Some(executable.parent()?.parent()?.join("Resources"))
}
