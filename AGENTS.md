# Polish

Polish is a macOS writing app. Rune renders the editor and Settings tab. Rust handles requests and Keychain access. The Swift host also registers the legacy macOS text service.

## Workflows

- Run `cargo fmt --check`, `cargo check`, and `cargo test --release` after Rust changes.
- Run `./scripts/install.sh` to build and install the current source in `~/Applications/Polish.app`.
- Add `--open` to open the installed app. Existing app processes keep their old code until reopened.
- Do not register the project build as the installed app. Duplicate registrations can prevent the service from launching correctly.

## Invariants

- Keep the service host and the settings app in separate bundles with distinct identifiers. macOS must be able to launch the service while the settings window is closed.
- Keep the host out of the Dock. The settings app may appear there.
- Keep API keys in Keychain. Do not log keys or selected text. Never include credentials in repository files.
- Send the complete document and exact selected passage to the model. Without a selection, use the whole document. Never apply a result to a document that changed after the request. Continue idea inserts one sentence after the selected passage. Each applied change must remain undoable.
- Preserve the original text on errors. The legacy service depends on app support and may lose rich formatting.
- Preserve reduced-motion behavior for the animated icon. Keep completion sounds quiet and optional.
- Rune is pre-release. The local Rune copy is based on revision `9587433` and adds selected-glyph motion. Keep its matching dependency patches pinned and test builds after dependency changes.
