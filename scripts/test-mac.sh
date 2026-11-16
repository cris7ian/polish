#!/bin/bash
# Compile the host and run deterministic fixtures without Keychain, Accessibility, or API calls.
set -euo pipefail
cd "$(dirname "$0")/.."
source scripts/swift-sources.sh
fixture_dir=$(mktemp -d "${TMPDIR:-/tmp}/polish-host-tests.XXXXXX")
cleanup() {
  if command -v trash >/dev/null 2>&1; then
    trash "$fixture_dir"
  else
    mkdir -p "${HOME:?}/.Trash"
    mv "$fixture_dir" "$HOME/.Trash/$(basename "$fixture_dir")-$$"
  fi
}
trap cleanup EXIT
swiftc -swift-version 5 mac/main.swift "${polish_swift_modules[@]}" -o "$fixture_dir/Polish" -framework AppKit
swiftc -swift-version 5 -D POLISH_INTEGRATION_TEST mac/main.swift "${polish_swift_modules[@]}" -o "$fixture_dir/Polish-integration" -framework AppKit
swiftc -swift-version 5 "${polish_swift_modules[@]}" scripts/test-selection-actions.swift -o "$fixture_dir/selection-tests" -framework AppKit
"$fixture_dir/selection-tests"
swiftc -swift-version 5 "${polish_swift_modules[@]}" scripts/test-host.swift -o "$fixture_dir/host-tests" -framework AppKit
"$fixture_dir/host-tests" "$fixture_dir"
