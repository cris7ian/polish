use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Serialize, Deserialize)]
pub(crate) struct Config {
    pub(crate) model: String,
    #[serde(default = "default_enabled")]
    pub(crate) sound: bool,
    #[serde(default = "default_enabled")]
    pub(crate) paste_on_startup: bool,
    #[serde(default = "default_enabled")]
    pub(crate) menu_bar_icon: bool,
}
fn default_enabled() -> bool {
    true
}
impl Default for Config {
    fn default() -> Self {
        Self {
            model: "gpt-6-luna".into(),
            sound: true,
            paste_on_startup: true,
            menu_bar_icon: true,
        }
    }
}
fn config_path() -> Result<PathBuf, String> {
    let home = std::env::var_os("HOME").ok_or("Home directory unavailable")?;
    Ok(PathBuf::from(home).join("Library/Application Support/Polish/config.json"))
}
pub(crate) fn load_config() -> Config {
    config_path()
        .ok()
        .and_then(|p| fs::read(p).ok())
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}
pub(crate) fn save_config(config: &Config) -> Result<(), String> {
    let path = config_path()?;
    fs::create_dir_all(path.parent().ok_or("Invalid config path")?).map_err(|e| e.to_string())?;
    let bytes = serde_json::to_vec_pretty(config).map_err(|e| e.to_string())?;
    fs::write(path, bytes).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn paste_on_startup_config_defaults_and_round_trips() {
        let old: Config = serde_json::from_str(r#"{"model":"example"}"#).unwrap();
        assert!(old.paste_on_startup);
        assert!(old.menu_bar_icon);
        assert!(Config::default().paste_on_startup);
        let disabled = Config {
            paste_on_startup: false,
            menu_bar_icon: false,
            ..Config::default()
        };
        let saved = serde_json::to_string(&disabled).unwrap();
        assert!(
            !serde_json::from_str::<Config>(&saved)
                .unwrap()
                .menu_bar_icon
        );
        assert!(
            !serde_json::from_str::<Config>(&saved)
                .unwrap()
                .paste_on_startup
        );
    }
    #[test]
    fn default_model_is_luna() {
        assert_eq!(Config::default().model, "gpt-6-luna");
    }
}
