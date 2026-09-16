#!/usr/bin/env bash
set -euo pipefail

version="${1:?version is required}"
root="${2:?asset directory is required}"

expected=(
  "bloody-level-$version-windows-x64.exe"
  "bloody-level-$version-windows-x64.msi"
  "bloody-level-$version-windows-x64.zip"
  "bloody-level-$version-windows-arm64.exe"
  "bloody-level-$version-windows-arm64.msi"
  "bloody-level-$version-windows-arm64.zip"
  "bloody-level-$version-linux-x64.AppImage"
  "bloody-level-$version-linux-x64.deb"
  "bloody-level-$version-linux-x64.rpm"
  "bloody-level-$version-linux-x64.flatpak"
  "bloody-level-$version-linux-x64.zip"
  "bloody-level-$version-linux-arm64.AppImage"
  "bloody-level-$version-linux-arm64.deb"
  "bloody-level-$version-linux-arm64.rpm"
  "bloody-level-$version-linux-arm64.flatpak"
  "bloody-level-$version-linux-arm64.zip"
  "bloody-level-$version-macos-x64.dmg"
  "bloody-level-$version-macos-x64.zip"
  "bloody-level-$version-macos-arm64.dmg"
  "bloody-level-$version-macos-arm64.zip"
)

for name in "${expected[@]}"; do
  mapfile -t matches < <(find "$root" -type f -name "$name" -print)
  if [[ "${#matches[@]}" -ne 1 ]]; then
    echo "Expected exactly one release asset named $name, found ${#matches[@]}" >&2
    exit 1
  fi
  if [[ ! -s "${matches[0]}" ]]; then
    echo "Release asset is empty: ${matches[0]}" >&2
    exit 1
  fi
done

mapfile -t actual < <(find "$root" -type f -print)
if [[ "${#actual[@]}" -ne "${#expected[@]}" ]]; then
  echo "Expected ${#expected[@]} release assets, found ${#actual[@]}" >&2
  printf '%s\n' "${actual[@]}" >&2
  exit 1
fi

printf 'Verified %d release assets for %s:\n' "${#expected[@]}" "$version"
printf '%s\n' "${expected[@]}"
