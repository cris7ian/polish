#!/bin/bash
# Build a Developer ID-signed disk image, or notarize an existing image.
set -euo pipefail
cd "$(dirname "$0")/.."

if [ "${1:-}" = "--appcast" ]; then
  if [ "$#" -ne 2 ] || [ ! -f "$2" ]; then
    echo 'Usage: ./scripts/release.sh --appcast path/to/notarized-image.dmg' >&2
    exit 2
  fi
  image="$(cd "$(dirname "$2")" && pwd -P)/$(basename "$2")"
  xcrun stapler validate "$image"
  spctl --assess --type open --context context:primary-signature "$image"
  ./scripts/sparkle.sh
  version=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' mac/Info.plist)
  archive_dir=$(mktemp -d "${TMPDIR:-/tmp}/polish-appcast.XXXXXX")
  cleanup_appcast() {
    if command -v trash >/dev/null 2>&1; then trash "$archive_dir" || true
    elif [ -d "$HOME/.Trash" ]; then mv "$archive_dir" "$HOME/.Trash/$(basename "$archive_dir")"
    else rm -r "$archive_dir"; fi
  }
  trap cleanup_appcast EXIT
  cp "$image" "$archive_dir/"
  build/Sparkle-2.10.1/bin/generate_appcast --account com.cristian.polish \
    --download-url-prefix "https://github.com/cris7ian/polish/releases/download/v${version}/" \
    --maximum-deltas 0 "$archive_dir"
  if [ -e build/distribution/appcast.xml ]; then
    echo 'Refusing to overwrite build/distribution/appcast.xml.' >&2
    exit 1
  fi
  cp "$archive_dir/appcast.xml" build/distribution/appcast.xml
  echo 'Created build/distribution/appcast.xml'
  exit 0
fi

if [ "${1:-}" = "--notarize" ]; then
  if [ "$#" -ne 2 ] || [ -z "${NOTARY_PROFILE:-}" ]; then
    echo 'Usage: NOTARY_PROFILE=profile ./scripts/release.sh --notarize path/to/Polish.dmg' >&2
    exit 2
  fi
  image="$2"
  codesign --verify --verbose=2 "$image"
  xcrun notarytool submit "$image" --keychain-profile "$NOTARY_PROFILE" --wait
  xcrun stapler staple "$image"
  xcrun stapler validate "$image"
  spctl --assess --type open --context context:primary-signature --verbose=2 "$image"
  exit 0
fi
if [ "$#" -ne 0 ]; then echo 'Usage: ./scripts/release.sh [--notarize image]' >&2; exit 2; fi

identity="${POLISH_SIGNING_IDENTITY:-}"
if [ -z "$identity" ]; then
  identity=$(security find-identity -v -p codesigning | awk '/"Developer ID Application:/ {print $2; exit}')
fi
if ! security find-identity -v -p codesigning | grep -F "$identity" | grep -q '"Developer ID Application:' || [ -z "$identity" ]; then
  echo 'A valid Developer ID Application identity is required.' >&2
  exit 1
fi
version=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' mac/Info.plist)
if [[ ! "$version" =~ ^[0-9]+(\.[0-9]+)*$ ]]; then echo 'Invalid version.' >&2; exit 1; fi
architecture=$(uname -m)
output="build/distribution/Polish-${version}-${architecture}.dmg"
if [ -e "$output" ]; then echo "Output already exists: $output" >&2; exit 1; fi
POLISH_RELEASE_SIGNING=1 POLISH_SIGNING_IDENTITY="$identity" ./scripts/build-app.sh
mkdir -p build/distribution
stage=$(mktemp -d "${TMPDIR:-/tmp}/polish-release.XXXXXX")
cleanup() {
  if command -v trash >/dev/null 2>&1; then trash "$stage" || true
  elif [ -d "$HOME/.Trash" ]; then mv "$stage" "$HOME/.Trash/$(basename "$stage")"
  else rm -r "$stage"; fi
}
trap cleanup EXIT
ditto build/Polish.app "$stage/Polish.app"
ln -s /Applications "$stage/Applications"
hdiutil create -volname Polish -srcfolder "$stage" -format UDZO "$output"
codesign --force --timestamp --sign "$identity" "$output"
codesign --verify --verbose=2 "$output"
printf 'Signed image: %s (not notarized)\n' "$output"
