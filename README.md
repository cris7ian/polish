# Polish

Polish is a macOS writing app. Paste or write a document, select a passage, and choose Rework, Polish, Shorten, or Continue idea (…). Polish sends the full document and the selected passage to OpenAI so the result fits the surrounding text. Without a selection, the action uses the full document. The action updates the passage in place and keeps it selected for another pass. Continue idea inserts one sentence after the passage and keeps the original passage selected. Use Undo to reverse a change, or Copy to copy the document.

The animated flag responds to your system's reduced-motion setting. Each action plays a different quiet note when its result is ready. A separate note signals a failed request. Turn sounds off in Settings.

## Install or update

Requirements: macOS, Rust and Cargo, Xcode command-line tools, and `rsvg-convert` from librsvg. Install librsvg with `brew install librsvg` if needed.

Run from the project directory:

```sh
./scripts/install.sh
```

The script builds the app and installs it at `~/Applications/Polish.app`. If Polish is open, quit and reopen it to use the new build. To open it after installation, run `./scripts/install.sh --open`.

Open Polish from Launchpad or Applications. In the Settings tab, enter an OpenAI API key, select a model, and save. The key is stored in macOS Keychain. A green indicator means OpenAI accepted the key; it does not confirm model access or billing. The copy button places the key on the clipboard for up to 30 seconds.

If **Polish** is missing from Services, enable it in **System Settings → Keyboard → Keyboard Shortcuts → Services → Text**. Reopen the app where you selected text if its menu does not refresh. You can assign a keyboard shortcut in the same settings panel.

## Privacy and behavior

The full document and selected passage are sent to OpenAI. API usage is billed separately from a ChatGPT subscription. The default model is GPT-6 Luna; model availability depends on the API account. The editor uses plain text. Polish never applies a result if the document changed after the request. You can undo each applied result. The legacy macOS service also remains available in compatible apps and may remove rich formatting.

## Development

```sh
cargo check
cargo test --release
./scripts/build-app.sh
```

The build is ad hoc signed for local use and is not notarized. Rune is pinned to a Git revision; its required patches are pinned in `Cargo.toml`. The app host provides the service. A separate settings app keeps its process identity distinct so macOS can start the service when the settings window is closed.

Configuration is stored in `~/Library/Application Support/Polish/config.json`; the API key is stored separately in Keychain. Do not add credentials or user configuration to the repository.
