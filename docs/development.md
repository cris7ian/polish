# Development

## Build and test

Requirements: macOS, Rust and Cargo, Xcode command-line tools, `curl`, and `rsvg-convert` from librsvg. Install librsvg with `brew install librsvg`.

```sh
cargo fmt --check
cargo check
cargo test --release
./scripts/test-mac.sh
./scripts/build-app.sh
```

The build downloads the pinned Sparkle distribution and verifies its checksum. To build and install locally, run `./scripts/install.sh`; add `--open` to open the installed app. Quit and reopen existing processes to load a new build.

The macOS host tests require neither API calls nor Accessibility access. For native selection tests, build with `POLISH_INTEGRATION_TEST=1 ./scripts/build-app.sh`. The host accepts `--integration-selection polish` or `--integration-selection reroll` and `--expected-source-bundle BUNDLE_ID`. Use `--without-ax` to exercise the Copy fallback or `--activate-host` to test source-app restoration. Keyboard events still require Accessibility permission. Release signing rejects integration-test builds.

Fixture tests do not prove native paste, Undo, or reselection in a source app. Verify those behaviors in a permitted app.

## Release

Update the version in `Cargo.toml`, `Cargo.lock`, and both macOS Info plists. Increase `CFBundleVersion` in both plists for every release. A Developer ID Application certificate, a notarization Keychain profile, and the Sparkle signing key are required.

```sh
./scripts/release.sh
NOTARY_PROFILE=profile ./scripts/release.sh --notarize build/distribution/Polish-VERSION-ARCH.dmg
./scripts/release.sh --appcast build/distribution/Polish-VERSION-ARCH.dmg
```

Replace `VERSION` and `ARCH` with the output filename. Publish the notarized disk image and `appcast.xml` together in the GitHub release. The feed contains the image signature and points to that release; the feed itself is not signed. Keep a secure backup of the Sparkle signing key. Update the Homebrew cask after publishing.

## Website

Preview the static site with:

```sh
python3 -m http.server 8765 --bind 127.0.0.1 --directory website
```

Run its checks:

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

Changes to `website/` on `main` trigger the website deployment workflow.

Configuration is stored in `~/Library/Application Support/Polish/config.json`; API keys are stored in Keychain. Do not add credentials or user configuration to the repository.
