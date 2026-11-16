const SERVICE: &str = "com.cristian.polish.openai";
const ACCOUNT: &str = "api-key";

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
