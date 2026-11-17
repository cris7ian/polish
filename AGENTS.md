# Polish

Polish is a macOS writing app. Rune renders the editor and Settings tab. Rust handles requests and Keychain access. The Swift host also registers the legacy macOS text service.

## Workflows

- Run `cargo fmt --check`, `cargo check`, and `cargo test --release` after Rust changes.
- Run `./scripts/test-mac.sh` after Swift host changes. It needs no API calls or Accessibility access.
- Verify menu selection changes in a real permitted app before claiming compatibility with that app. Fixture tests do not establish native paste or Undo behavior.
- `POLISH_INTEGRATION_TEST=1 ./scripts/build-app.sh` enables the controlled native-test entry point. Production builds omit it; release signing rejects it.
- Run `./scripts/install.sh` to build and install the current source in `~/Applications/Polish.app`.
- Add `--open` to open the installed app. Existing app processes keep their old code until reopened.
- Do not register the project build as the installed app. Duplicate registrations can prevent the service from launching correctly.
- Publish the notarized disk image and `appcast.xml` containing its Sparkle signature together. Increment `CFBundleVersion` in both app bundles for every release.

## Invariants

- Keep the service host and the settings app in separate bundles with distinct identifiers. macOS must be able to launch the service while the settings window is closed.
- Keep the host out of the Dock. The settings app may appear there.
- Keep API keys in Keychain. Do not log keys or selected text. Never include credentials in repository files.
- Send the complete document and exact selected passage when available. Menu actions must fall back to selected text if the source app does not expose the document. Without a selection, use the whole document if available.
- The editor rejects results if its document changed during the request. Menu actions recheck the source app and all available text, selection, and focus information. Skip native paste when an observed source change occurs.
- Accessibility document and focus information is optional context for menu actions; native Copy provides selection-only fallback. Selection-only capture must never select or replace the whole document.
- Menu actions replace the clipboard with the result, then attempt native Paste. Keep copied-result fallback quiet in the menu status. Do not restore earlier clipboard contents. Preserve native Undo and reselect the replacement where supported.
- Continue idea inserts one sentence after the selected passage. Each applied editor change must remain undoable.
- Preserve the original text on errors. The legacy service depends on app support and may lose rich formatting.
- Keep menu action completion sounds matched to their editor actions, after copying the result.
- Preserve reduced-motion behavior for the animated icon. Keep completion sounds quiet and optional. The Swift service reads the saved sound setting after each successful request and plays the host bundle’s `ready.wav` only after returning text to the service pasteboard.
- Rune is pre-release. The local Rune copy is based on revision `9587433` and adds selected-glyph motion. Keep its matching dependency patches pinned and test builds after dependency changes.

## Website

- Preview with `python3 -m http.server 8765 --bind 127.0.0.1 --directory website`.
- Check with `python3 scripts/verify-website.py`, `node --check website/water.js`, `node --check website/demo.js`, `node --check website/app-demo.js`, `node --check website/text-motion.js`, `node --check website/homebrew-copy.js`, `node scripts/test-website-demo.mjs`, and `node scripts/test-website-homebrew.mjs`.
- Keep the hero waterline around half-height, with visible water around the narrow demo card.
- Preserve the intentional starting typo “pasghetti” in the hero demo; it is a Simpsons reference.
- Keep the hero demo limited to typo correction and rerolling one word. It uses fixed text and makes no API requests.
- Autoplay stays silent until a demo interaction unlocks audio. Reduced motion disables autoplay; manual actions still work.
- `website/polish-app.png` is the real screenshot in the second panel. Keep it separate from the abstract hero demo. Its non-clickable text overlay in `website/app-demo.js` animates once on visibility and stays static with reduced motion.
- Website sound files are copies of `assets/sounds/ready.wav` and `assets/sounds/rework.wav`; update both copies when changing these sounds.

- Selected text uses per-letter vertical motion from Rune’s `vendor/rune/src/runtime/paint/content.rs`, not whole-word rotation. `website/text-motion.js` shares that wave between demos; the hero increases its amplitude. Keep the selection background still and honor reduced motion.
