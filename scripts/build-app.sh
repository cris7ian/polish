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
cp target/release/polish build/Polish.app/Contents/MacOS/polish-core
mkdir -p "build/Polish.app/Contents/Helpers/Polish Settings.app/Contents/MacOS" "build/Polish.app/Contents/Helpers/Polish Settings.app/Contents/Resources"
cp mac/Settings-Info.plist "build/Polish.app/Contents/Helpers/Polish Settings.app/Contents/Info.plist"
cp target/release/polish "build/Polish.app/Contents/Helpers/Polish Settings.app/Contents/MacOS/polish-settings"
cp build/Polish.app/Contents/Resources/Polish.icns "build/Polish.app/Contents/Helpers/Polish Settings.app/Contents/Resources/Polish.icns"
cp assets/sounds/*.wav "build/Polish.app/Contents/Helpers/Polish Settings.app/Contents/Resources/"
swift scripts/render-symbols.swift "build/Polish.app/Contents/Helpers/Polish Settings.app/Contents/Resources/Symbols"
swiftc -swift-version 5 mac/main.swift -o build/Polish.app/Contents/MacOS/Polish -framework AppKit
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
codesign --force --sign "$signing_identity" --identifier com.cristian.polish.core build/Polish.app/Contents/MacOS/polish-core
codesign --force --sign "$signing_identity" "build/Polish.app/Contents/Helpers/Polish Settings.app"
codesign --force --sign "$signing_identity" build/Polish.app
codesign --verify --deep --strict build/Polish.app
echo "Built build/Polish.app"
