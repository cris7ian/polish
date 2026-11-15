# Polish

Polish is a macOS selected-text service. Rust handles settings, API requests, and Keychain access. Rune renders the settings window. The Swift host registers the macOS service.

## Workflows

- Run `cargo fmt --check`, `cargo check`, and `cargo test --release` after Rust changes.
- Run `./scripts/install.sh` to build and install the current source in `~/Applications/Polish.app`.
- Add `--open` to open the installed app. Existing app processes keep their old code until reopened.
- Do not register the project build as the installed app. Duplicate registrations can prevent the service from launching correctly.

## Invariants

- Keep the service host and the settings app in separate bundles with distinct identifiers. macOS must be able to launch the service while the settings window is closed.
- Keep the host out of the Dock. The settings app may appear there.
- Keep API keys in Keychain. Do not log keys or selected text. Never include credentials in repository files.
- Keep original selected text unchanged on errors. Account for apps that do not support text replacement and for loss of rich formatting.
- Preserve reduced-motion behavior for the animated icon.
- Rune is pre-release and pinned with matching workspace patches. Test builds after dependency changes.
