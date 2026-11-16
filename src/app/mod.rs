use crate::{
    config::{Config, load_config, save_config},
    credentials,
    editing::{Action, Target, WorkResult},
    engine::{KeyStatus, run_action, verify_key},
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
}
impl Default for State {
    fn default() -> Self {
        Self::new(load_config(), credentials::read().is_ok())
    }
}
impl State {
    fn new(cfg: Config, key_saved: bool) -> Self {
        let status = if key_saved {
            "API key saved in Keychain"
        } else {
            "Add an OpenAI API key to get started"
        };
        Self {
            key: TextEdit::default().secure(),
            model: TextEdit::new(&cfg.model),
            status: status.into(),
            key_saved,
            key_status: if key_saved {
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
    let result = (|| {
        if !s.key.text.trim().is_empty() {
            credentials::write(s.key.text.trim())?;
        }
        save_config(&Config {
            model: model.into(),
            sound: s.sound,
            paste_on_startup: s.paste_on_startup,
            menu_bar_icon: s.menu_bar_icon,
        })
    })();
    match result {
        Ok(()) => {
            s.key_saved = s.key_saved || !s.key.text.trim().is_empty();
            s.key = TextEdit::default().secure();
            s.status = "Settings saved".into();
            if s.key_saved {
                s.key_status = KeyStatus::Checking;
                s.key_check.request((), |_| verify_key());
            }
        }
        Err(e) => s.status = format!("Could not save: {e}"),
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
    if !s.key_saved {
        s.work_status = "Add an API key in Settings first.".into();
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
        if s.key_saved {
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
