use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

/// A registered Sign in with ChatGPT client for this host. Tokens stay in
/// Keychain; only this non-secret mapping is persisted here.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct Registration {
    pub(crate) client_id: String,
    #[serde(default)]
    pub(crate) email: String,
    #[serde(default)]
    pub(crate) subject: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Config {
    pub(crate) model: String,
    #[serde(default = "default_enabled")]
    pub(crate) sound: bool,
    #[serde(default = "default_enabled")]
    pub(crate) paste_on_startup: bool,
    #[serde(default = "default_enabled")]
    pub(crate) menu_bar_icon: bool,
    /// "api-key" or "chatgpt".
    #[serde(default = "default_auth")]
    pub(crate) auth: String,
    /// Stable host identifier sent on every Sign in with ChatGPT attempt.
    #[serde(default)]
    pub(crate) host_id: Option<String>,
    #[serde(default)]
    pub(crate) chatgpt_registration: Option<Registration>,
}
fn default_enabled() -> bool {
    true
}
fn default_auth() -> String {
    "api-key".into()
}
impl Default for Config {
    fn default() -> Self {
        Self {
            model: "gpt-6-luna".into(),
            sound: true,
            paste_on_startup: true,
            menu_bar_icon: true,
            auth: default_auth(),
            host_id: None,
            chatgpt_registration: None,
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
        assert_eq!(old.auth, "api-key");
        assert!(old.chatgpt_registration.is_none());
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
    #[test]
    fn chatgpt_config_round_trips_and_defaults_to_api_key() {
        let saved = serde_json::to_string(&Config {
            auth: "chatgpt".into(),
            host_id: Some("host-123".into()),
            chatgpt_registration: Some(Registration {
                client_id: "oaiapp_1".into(),
                email: "user@example.com".into(),
                subject: "sub-1".into(),
            }),
            ..Config::default()
        })
        .unwrap();
        let loaded: Config = serde_json::from_str(&saved).unwrap();
        assert_eq!(loaded.auth, "chatgpt");
        assert_eq!(loaded.host_id.as_deref(), Some("host-123"));
        assert_eq!(
            loaded.chatgpt_registration,
            Some(Registration {
                client_id: "oaiapp_1".into(),
                email: "user@example.com".into(),
                subject: "sub-1".into(),
            })
        );
        // Partial records keep older installs loadable.
        let partial: Config = serde_json::from_str(r#"{"model":"gpt-6-luna"}"#).unwrap();
        assert_eq!(partial.auth, "api-key");
        assert!(partial.host_id.is_none());
    }
}
