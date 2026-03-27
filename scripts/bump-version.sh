#!/usr/bin/env bash
# Usage: ./scripts/bump-version.sh 0.2.0
set -euo pipefail

VERSION="${1:?Usage: bump-version.sh <version>}"

echo "Bumping to $VERSION"

# Root package.json
sed -i '' "s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" package.json

# Tauri config
sed -i '' "s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" src-tauri/tauri.conf.json

# Cargo.toml (first version line only)
sed -i '' "0,/^version = \".*\"/s//version = \"$VERSION\"/" src-tauri/Cargo.toml

echo "Done. Verify with:"
echo "  grep '\"version\"' package.json src-tauri/tauri.conf.json"
echo "  head -3 src-tauri/Cargo.toml"
