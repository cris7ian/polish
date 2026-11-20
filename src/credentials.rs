const SERVICE: &str = "com.cristian.polish.openai";
const ACCOUNT: &str = "api-key";
const OAUTH_ACCOUNT: &str = "chatgpt-oauth";

fn entry() -> Result<keyring::Entry, String> {
    keyring::Entry::new(SERVICE, ACCOUNT).map_err(|e| e.to_string())
}

pub(crate) fn read() -> Result<String, String> {
    entry()?.get_password().map_err(|e| e.to_string())
}

pub(crate) fn write(key: &str) -> Result<(), String> {
    entry()?.set_password(key).map_err(|e| e.to_string())
}

pub(crate) fn remove() -> Result<(), String> {
    entry()?.delete_credential().map_err(|e| e.to_string())
}

fn oauth_entry() -> Result<keyring::Entry, String> {
    keyring::Entry::new(SERVICE, OAUTH_ACCOUNT).map_err(|e| e.to_string())
}

/// The Sign in with ChatGPT credential record, stored as JSON.
pub(crate) fn read_oauth() -> Result<String, String> {
    oauth_entry()?.get_password().map_err(|e| e.to_string())
}

pub(crate) fn write_oauth(record: &str) -> Result<(), String> {
    oauth_entry()?
        .set_password(record)
        .map_err(|e| e.to_string())
}

pub(crate) fn remove_oauth() -> Result<(), String> {
    oauth_entry()?
        .delete_credential()
        .map_err(|e| e.to_string())
}
