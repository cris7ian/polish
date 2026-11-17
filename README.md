# Polish

Polish is a macOS writing app. Paste or write a document, select a passage, and choose Rework, Polish, Shorten, or Continue idea (…). Polish sends the full document and the selected passage to OpenAI so the result fits the surrounding text. Polish Tweet rewrites the selection as a post targeting 280 characters; it rejects results over 280 Unicode characters, though X's weighted character counting may differ. Without a selection, the action uses the full document. The action updates the passage in place and keeps it selected for another pass. Continue idea inserts one sentence after the passage and keeps the original passage selected. Use Undo to reverse a change, or Copy to copy the document.

Keyboard shortcuts: ⌘R for Rework, ⌘P for Polish, ⌘. for Shorten, ⌘T for Polish Tweet, ⌘Z for Undo, and ⌘⇧Z for Redo. The right-click Services menu also offers Polish Tweet.

On launch, the editor pastes plain text from the clipboard into an empty document. Turn off **Paste on startup** in Settings to disable this. The trash button clears the document; use Undo to restore it. Clipboard contents are not sent to OpenAI until you choose an editing action.

Turn **menu bar icon** on or off in Settings, then save to show or hide the flag. The menu bar flag opens Polish from **Open Polish**. Choose **Quit Polish** to close the editor and service host. The flag adapts to light and dark menu bars, with a translucent upper stripe and a solid lower stripe.

Select text in another app, then choose **Polish** or **Reroll** from the menu bar flag. The first use requires Accessibility access in **System Settings → Privacy & Security → Accessibility**. These actions use the complete document when the source app exposes it. Otherwise, they use the selected text. If needed, Polish reads the selection with the source app’s Copy command. Without a selection, they use the whole document only when it is available.

Polish corrects spelling, grammar, and punctuation. Reroll uses the editor's Rework action to rewrite a selected word or passage for clarity and flow. Both actions send the available context and exact selected passage to OpenAI. Each action copies its result and attempts a native paste after rechecking the available text, selection, and focus. Native Copy and result copying replace existing clipboard contents. Use the source app's Undo command to reverse the paste. Where supported, the replacement stays selected for another reroll.

If Polish detects a source change, it skips the paste and leaves the result on the clipboard. If it cannot confirm the paste, the menu quietly shows **Result copied; paste attempted**. Paste manually if needed. Apps expose different amounts of selection and document information, so automatic checks and reselection depend on the source app. Request errors preserve the source text.

Click the animated flag to disturb the water. Its waves settle after each click and respect your system's reduced-motion setting. Each action plays a different quiet note when its result is ready. A separate note signals a failed request. The right-click Polish service also plays the completion note after it returns the corrected text. The menu bar actions play the same Polish and Rework completion notes after copying their results. Turn sounds off in Settings to silence the editor, menu bar actions, and service.

## Install or update

Requirements: macOS, Rust and Cargo, Xcode command-line tools, and `rsvg-convert` from librsvg. Install librsvg with `brew install librsvg` if needed.

Run from the project directory:

```sh
./scripts/install.sh
```

The script builds the app and installs it at `~/Applications/Polish.app`. If Polish is open, quit and reopen it to use the new build. To open it after installation, run `./scripts/install.sh --open`.

Open Polish from Launchpad or Applications. In the Settings tab, enter an OpenAI API key, select a model, and save. The key is stored in macOS Keychain. A green indicator means OpenAI accepted the key; it does not confirm model access or billing. The copy button places the key on the clipboard for up to 30 seconds.

If **Polish** or **Polish Tweet** is missing from Services, enable it in **System Settings → Keyboard → Keyboard Shortcuts → Services → Text**. macOS may require enabling each service separately. Reopen the app where you selected text if its menu does not refresh. You can assign a keyboard shortcut in the same settings panel.

## Privacy and behavior

Editor actions send the full document and selected passage to OpenAI. Menu bar actions send the available document context, or selected text alone when full context is unavailable. Legacy Services send the text supplied by the source app. API usage is billed separately from a ChatGPT subscription. The default model is GPT-6 Luna; model availability depends on the API account.

The editor uses plain text and rejects a result if its document changed during the request. Menu actions reject changes they can detect from the source app's available information. Applied editor changes and native pastes support Undo. The legacy macOS service remains available in compatible apps and may remove rich formatting.

## Development

```sh
cargo fmt --check
cargo check
cargo test --release
./scripts/build-app.sh
```

Run the deterministic macOS host checks without API calls or Accessibility access:

```sh
./scripts/test-mac.sh
```

These checks cover Unicode selection ranges, stale documents, selection-only context, clipboard capture, source-app tracking, passage replacement, and request encoding. They also cover helper startup, large input, concurrent output, early exits, timeouts, and preference defaults. The script compiles both production and integration-test host variants. Verify Accessibility permission, native paste, and Undo in the source apps you support.

For controlled native integration tests, build with `POLISH_INTEGRATION_TEST=1 ./scripts/build-app.sh`. The host then accepts `--integration-selection polish` or `--integration-selection reroll`, with `--expected-source-bundle BUNDLE_ID` matching the frontmost test app. `--without-ax` disables Accessibility text and focus capture to exercise native Copy fallback; Accessibility permission is still required for keyboard events. `--activate-host` tests restoring the source app after host activation. Production builds omit this entry point, and release signing rejects integration-test builds.

Rust code separates pure editing rules in `src/editing.rs` from OpenAI requests in `src/engine.rs`. `src/credentials.rs` owns Keychain access; `src/config.rs` owns saved settings. `src/app/mod.rs` owns editor state and document updates. Its private `ui/` modules render the editor, settings, and animated logo. `src/main.rs` selects the app or command-line entry point.

The Swift host keeps menu wiring in `mac/main.swift`. Both Services and menu actions use `CoreProcess.swift` to run the Rust helper. `HostPreferences.swift` reads and observes settings for the menu and `CompletionSound.swift`. `SelectionSnapshot.swift` validates captured text and chooses full-document or selection-only context. `SelectionClipboard.swift` reads selected text through the source app’s Copy command. `SelectionSource.swift` tracks the last external app independently of optional Accessibility focus information. `SelectionActions.swift` coordinates capture, requests, and native paste. Builds and tests share the module list in `scripts/swift-sources.sh`.

The local build uses an available signing identity, or ad hoc signing if none is available. The local Rune copy is based on revision `9587433` and adds selected-glyph motion. Its required dependency patches are pinned in `Cargo.toml`. The app host provides the service. A separate settings app keeps its process identity distinct so macOS can start the service when the settings window is closed.

## Release

To prepare a disk image for the current Mac architecture, install a Developer ID Application certificate and run `./scripts/release.sh`. The script signs the app and disk image with a secure timestamp. It writes the image to `build/distribution/`.

Before distribution, store Apple notarization credentials in a Keychain profile. Then run `NOTARY_PROFILE=profile ./scripts/release.sh --notarize build/distribution/Polish-1.2.0-arm64.dmg`, replacing the profile and filename as needed. The script submits the image, staples the ticket, and checks Gatekeeper acceptance. Apple may require the account holder to accept current agreements before notarization.

Generate the signed update feed with `./scripts/release.sh --appcast build/distribution/Polish-1.2.0-arm64.dmg`. Publish `appcast.xml` with the notarized image in the GitHub release. Keep the Sparkle signing key in Keychain and maintain a secure backup. Increase `CFBundleVersion` in both bundles for every release.

## Website

The static site is in `website/` and needs no build step. Preview it locally:

```sh
python3 -m http.server 8765 --bind 127.0.0.1 --directory website
```

Open `http://127.0.0.1:8765/`. The hero keeps the waterline around half-height, with red water below and visible water around the narrow card. It demonstrates two fixed edits: correcting typos, then rerolling one word. It runs once and offers replay. It does not call OpenAI. Autoplay is silent; interacting with the demo enables quiet app sounds. Reduced-motion users advance the demo manually. The second panel pairs a creator note with a non-clickable app screenshot. Its text overlay selects and corrects the original passage, starting with “Teh pasghetti,” then rerolls “delicious” through “divine,” “incredible,” “heavenly,” and “unforgettable.” Selected letters follow the app’s vertical wave while the selection background stays still. The hero uses a larger wave and a staggered settling animation. This silent animation runs once when the screenshot comes into view and pauses offscreen. Reduced motion shows a static corrected passage.

Run the website checks from the project directory:

```sh
python3 scripts/verify-website.py
node --check website/water.js
node --check website/demo.js
node --check website/app-demo.js
node --check website/text-motion.js
node --check website/homebrew-copy.js
node scripts/test-website-demo.mjs
node scripts/test-website-homebrew.mjs
```

The checks cover local HTML assets and metadata, plus demo sequencing, replay, visibility, silent autoplay, and reduced motion. Changes to the site on `main` trigger the website deployment workflow. It uses a GitHub OpenID Connect role to publish to the private `polish.salsaparapizza.com` S3 bucket and invalidate CloudFront distribution `E1XDVY8I9CM0OV`. The intended public address is `https://polish.salsaparapizza.com/`.

Configuration is stored in `~/Library/Application Support/Polish/config.json`; the API key is stored separately in Keychain. Do not add credentials or user configuration to the repository.
