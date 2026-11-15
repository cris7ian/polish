#!/bin/bash
# Build and install Polish for the current macOS user.
set -euo pipefail
cd "$(dirname "$0")/.."
project_dir="$(pwd -P)"
applications_dir="${HOME:?}/Applications"
destination="$applications_dir/Polish.app"
register="/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister"
open_after=false
case "${1:-}" in
  '') ;;
  --open) open_after=true ;;
  *) echo "Usage: scripts/install.sh [--open]" >&2; exit 2 ;;
esac
if [ "$#" -gt 1 ]; then echo "Usage: scripts/install.sh [--open]" >&2; exit 2; fi
for command in cargo swiftc rsvg-convert iconutil codesign ditto; do
  if ! command -v "$command" >/dev/null 2>&1; then echo "Missing required command: $command" >&2; exit 1; fi
done
if [ -e "$destination" ]; then
  bundle_id=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' "$destination/Contents/Info.plist" 2>/dev/null || true)
  if [ "$bundle_id" != 'com.cristian.polish' ]; then
    echo "Refusing to replace $destination: it is not the Polish app." >&2; exit 1
  fi
fi
"$project_dir/scripts/build-app.sh"
mkdir -p "$applications_dir"
stage=$(mktemp -d "$applications_dir/.polish-install.XXXXXX")
completed=false
cleanup() {
  if [ "$completed" = false ] && [ -d "$stage/previous.app" ]; then
    if [ -e "$destination" ]; then mv "$destination" "$stage/failed.app"; fi
    mv "$stage/previous.app" "$destination"
    "$register" -f "$destination" || true
  fi
  if command -v trash >/dev/null 2>&1; then trash "$stage" || true; else rm -r "$stage"; fi
}
trap cleanup EXIT

ditto "$project_dir/build/Polish.app" "$stage/Polish.app"
codesign --verify --deep --strict "$stage/Polish.app"
if [ -e "$destination" ]; then mv "$destination" "$stage/previous.app"; fi
mv "$stage/Polish.app" "$destination"
"$register" -u "$project_dir/build/Polish.app" >/dev/null 2>&1 || true
"$register" -f "$destination"
completed=true
printf 'Installed %s\n' "$destination"
printf 'If Polish was already open, quit and reopen it to use this build.\n'
if "$open_after"; then open "$destination"; fi
