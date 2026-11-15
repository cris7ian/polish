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
swiftc -swift-version 5 mac/main.swift -o build/Polish.app/Contents/MacOS/Polish -framework AppKit
codesign --force --sign - "build/Polish.app/Contents/Helpers/Polish Settings.app"
codesign --force --deep --sign - build/Polish.app
echo "Built build/Polish.app"
