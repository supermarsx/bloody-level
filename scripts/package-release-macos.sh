#!/usr/bin/env bash
set -euo pipefail

version="${1:?version is required}"
architecture="${2:?architecture is required}"
target_directory="${3:?target directory is required}"
output_directory="${4:?output directory is required}"

case "$architecture" in
  x64) target="x86_64-apple-darwin" ;;
  arm64) target="aarch64-apple-darwin" ;;
  *) echo "Unsupported macOS architecture: $architecture" >&2; exit 1 ;;
esac

bundle_directory="$target_directory/$target/release/bundle"
app_directory="$bundle_directory/macos"
dmg_directory="$bundle_directory/dmg"
app_path="$(find "$app_directory" -maxdepth 1 -type d -name '*.app' -print -quit)"
dmg_path="$(find "$dmg_directory" -maxdepth 1 -type f -name '*.dmg' -print -quit)"

if [[ -z "$app_path" || -z "$dmg_path" ]]; then
  echo "Expected one .app and one .dmg under $bundle_directory" >&2
  exit 1
fi

mkdir -p "$output_directory"
prefix="bloody-level-$version-macos-$architecture"
cp "$dmg_path" "$output_directory/$prefix.dmg"
ditto -c -k --sequesterRsrc --keepParent "$app_path" "$output_directory/$prefix.zip"

echo "Packaged $prefix.dmg and $prefix.zip"
