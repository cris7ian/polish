#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --release
mkdir -p build/Polish.app/Contents/MacOS
cp mac/Info.plist build/Polish.app/Contents/Info.plist
mkdir -p build/Polish.app/Contents/Resources build/Polish.iconset
for size in 16 32 128 256 512; do
  rsvg-convert -w "$size" -h "$size" assets/polish-logo.svg > "build/Polish.iconset/icon_${size}x${size}.png"
  double=$((size * 2))
  rsvg-convert -w "$double" -h "$double" assets/polish-logo.svg > "build/Polish.iconset/icon_${size}x${size}@2x.png"
done
iconutil -c icns build/Polish.iconset -o build/Polish.app/Contents/Resources/Polish.icns
cp assets/sounds/ready.wav assets/sounds/rework.wav build/Polish.app/Contents/Resources/
# Remove the obsolete service executable from builds made before the service shared the editor binary.
rm -f build/Polish.app/Contents/MacOS/polish-core
mkdir -p "build/Polish.app/Contents/Helpers/Polish Settings.app/Contents/MacOS" "build/Polish.app/Contents/Helpers/Polish Settings.app/Contents/Resources"
cp mac/Settings-Info.plist "build/Polish.app/Contents/Helpers/Polish Settings.app/Contents/Info.plist"
# Remove the old executable name from incremental builds.
rm -f "build/Polish.app/Contents/Helpers/Polish Settings.app/Contents/MacOS/polish-settings"
cp target/release/polish "build/Polish.app/Contents/Helpers/Polish Settings.app/Contents/MacOS/Polish"
cp build/Polish.app/Contents/Resources/Polish.icns "build/Polish.app/Contents/Helpers/Polish Settings.app/Contents/Resources/Polish.icns"
cp assets/sounds/*.wav "build/Polish.app/Contents/Helpers/Polish Settings.app/Contents/Resources/"
swift scripts/render-symbols.swift "build/Polish.app/Contents/Helpers/Polish Settings.app/Contents/Resources/Symbols"
source scripts/swift-sources.sh
polish_swift_flags=()
if [ "${POLISH_INTEGRATION_TEST:-0}" = 1 ]; then
  if [ "${POLISH_RELEASE_SIGNING:-0}" = 1 ]; then
    echo 'Integration test entry cannot be included in a release build.' >&2
    exit 1
  fi
  polish_swift_flags=(-D POLISH_INTEGRATION_TEST)
fi
swiftc -swift-version 5 ${polish_swift_flags[@]+"${polish_swift_flags[@]}"} mac/main.swift "${polish_swift_modules[@]}" -o build/Polish.app/Contents/MacOS/Polish -framework AppKit
# A stable signing identity lets macOS recognize the app across rebuilds.
# Set POLISH_SIGNING_IDENTITY to choose a specific identity, or '-' for ad hoc signing.
signing_identity="${POLISH_SIGNING_IDENTITY:-}"
if [ -z "$signing_identity" ]; then
  signing_identity=$(security find-identity -v -p codesigning | awk '/"Developer ID Application:/ {print $2; exit}')
  if [ -z "$signing_identity" ]; then
    signing_identity=$(security find-identity -v -p codesigning | awk '/"Apple Development:/ {print $2; exit}')
  fi
fi
if [ -z "$signing_identity" ]; then
  signing_identity='-'
  echo 'No signing identity found; using ad hoc signing. Keychain may prompt after rebuilds.' >&2
fi
signing_options=()
if [ "${POLISH_RELEASE_SIGNING:-0}" = 1 ]; then
  if [ "$signing_identity" = '-' ]; then echo 'A Developer ID signing identity is required for release.' >&2; exit 1; fi
  signing_options=(--timestamp --options runtime)
fi
codesign --force --sign "$signing_identity" ${signing_options[@]+"${signing_options[@]}"} "build/Polish.app/Contents/Helpers/Polish Settings.app"
codesign --force --sign "$signing_identity" ${signing_options[@]+"${signing_options[@]}"} build/Polish.app
codesign --verify --deep --strict build/Polish.app
echo "Built build/Polish.app"
