---
name: polish-release
description: Prepare, verify, and publish Polish macOS releases and update its Homebrew cask. Use when asked to build a release installer or release or publish Polish.
---

# Polish release

Read `AGENTS.md`, `README.md`, and `scripts/release.sh` first. Preserve unrelated work and do not commit, push, or create a tag without explicit user intent.

1. Inspect the worktree, current version, release history, and existing tags and artifacts. Respect an explicit version or version increment; otherwise present patch, minor, and major choices neutrally and confirm the version.
2. Update the Polish version in `Cargo.toml`, `Cargo.lock`, and both macOS Info plists. Increase `CFBundleVersion` in both plists. Run the applicable checks in `AGENTS.md` and verify the service works with the editor closed when service behavior changes.
3. Run `./scripts/release.sh` to build a Developer ID-signed DMG for the current architecture. Do not overwrite an existing image. The output is not notarized yet.
4. Use a configured Keychain notarization profile with `NOTARY_PROFILE=profile ./scripts/release.sh --notarize build/distribution/Polish-VERSION-ARCH.dmg`. If the environment has no profile, check `~/Developer/mana/.env.release.local`: source it in a subshell with `set -a` to export its `NOTARY_PROFILE`, without printing the file or credentials. Verify Apple accepts the submission, stapling succeeds, and Gatekeeper accepts the DMG. If Apple returns an error after issuing a submission ID, check that ID with `xcrun notarytool info` rather than submitting another copy. After acceptance, run `xcrun stapler staple`, `xcrun stapler validate`, and `spctl --assess --type open --context context:primary-signature` on the image. Ask for a profile only if none is available. Do not publish an unnotarized image as a finished release.
5. Generate the feed with `./scripts/release.sh --appcast build/distribution/Polish-VERSION-ARCH.dmg`. The script refuses to overwrite an existing feed; preserve an older `build/distribution/appcast.xml` before generating the new one. It contains the image signature; the feed itself is not signed. Keep the Sparkle signing key safe and maintain a secure backup.
6. When explicitly authorized, commit the intended changes, create the version tag, push, and publish the notarized DMG and `build/distribution/appcast.xml` as a GitHub release. Verify the published assets and digest. Do not force-push or replace an existing tag or release.

## Homebrew cask

After the matching GitHub release is public, update `~/Developer/homebrew-tap/Casks/polish.rb` in the `cris7ian/homebrew-tap` repository. Preserve any unrelated tap changes. Set the version and SHA-256 from the exact stapled release asset; confirm the published GitHub asset digest matches `shasum -a 256` of the local DMG. Keep the architecture restriction consistent with the release artifacts (currently arm64 only), the macOS minimum, and `app "Polish.app"`.

Validate the cask and fetch/check its artifact without replacing an installed Polish app. Ask before committing or pushing tap changes unless explicitly authorized. Once published, verify `brew audit --cask --strict cris7ian/tap/polish` and `brew fetch --cask cris7ian/tap/polish`. The install command is `brew install --cask cris7ian/tap/polish`; avoid installing a second copy alongside the existing user-installed app because duplicate registrations can disrupt the text service.

Report the release URL, DMG path, notarization and cask verification, and any unpublished changes or blockers.
