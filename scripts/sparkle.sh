#!/bin/bash
# Fetch the pinned Sparkle distribution and verify it before use.
set -euo pipefail
cd "$(dirname "$0")/.."
version=2.10.1
digest=1a856d2a9edb8e628de88121d7be04cf0e2167f8641650b78fca00b14244cf27
archive="build/Sparkle-${version}.tar.xz"
destination="build/Sparkle-${version}"
mkdir -p build
if [ ! -f "$archive" ]; then
  curl --fail --location --silent --show-error \
    "https://github.com/sparkle-project/Sparkle/releases/download/${version}/Sparkle-${version}.tar.xz" \
    -o "$archive"
fi
if [ "$(shasum -a 256 "$archive" | awk '{print $1}')" != "$digest" ]; then
  echo "Sparkle archive checksum mismatch: $archive" >&2
  exit 1
fi
if [ ! -d "$destination/Sparkle.framework" ]; then
  mkdir -p "$destination"
  tar -xJf "$archive" -C "$destination"
fi
