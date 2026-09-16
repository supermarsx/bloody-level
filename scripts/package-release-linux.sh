#!/usr/bin/env bash
set -euo pipefail

version="${1:?version is required}"
architecture="${2:?architecture is required}"
target_directory="${3:?target directory is required}"
output_directory="${4:?output directory is required}"

case "$architecture" in
  x64|arm64) ;;
  *) echo "Unsupported Linux architecture: $architecture" >&2; exit 1 ;;
esac

bundle_directory="$target_directory/release/bundle"
mkdir -p "$output_directory"
output_directory="$(realpath "$output_directory")"
prefix="bloody-level-$version-linux-$architecture"

find_one() {
  local directory="$1"
  local pattern="$2"
  mapfile -t matches < <(find "$directory" -maxdepth 1 -type f -name "$pattern" -print | sort)
  if [[ "${#matches[@]}" -ne 1 ]]; then
    echo "Expected exactly one $pattern in $directory, found ${#matches[@]}" >&2
    exit 1
  fi
  printf '%s\n' "${matches[0]}"
}

appimage="$(find_one "$bundle_directory/appimage" '*.AppImage')"
deb="$(find_one "$bundle_directory/deb" '*.deb')"
rpm="$(find_one "$bundle_directory/rpm" '*.rpm')"

appimage_output="$output_directory/$prefix.AppImage"
cp "$appimage" "$appimage_output"
cp "$deb" "$output_directory/$prefix.deb"
cp "$rpm" "$output_directory/$prefix.rpm"

zip_staging="$(mktemp -d)"
trap 'rm -rf "$zip_staging"' EXIT
cp "$appimage_output" "$zip_staging/$prefix.AppImage"
(cd "$zip_staging" && zip -q -9 "$output_directory/$prefix.zip" "$prefix.AppImage")

flatpak_input="$zip_staging/flatpak-input"
flatpak_build="$zip_staging/flatpak-build"
flatpak_repo="$zip_staging/flatpak-repo"
mkdir -p "$flatpak_input"
cp "$target_directory/release/blevel-tracker" "$flatpak_input/blevel-tracker"
cp "$target_directory/release/libpdfium.so" "$flatpak_input/libpdfium.so"
cp ontology/analytes.seed.json "$flatpak_input/analytes.seed.json"
cp src-tauri/icons/128x128.png "$flatpak_input/com.blevel.tracker.png"

cat > "$flatpak_input/com.blevel.tracker.desktop" <<'DESKTOP'
[Desktop Entry]
Name=bloody-level
Comment=Local pathology-report tracker
Exec=blevel-tracker
Icon=com.blevel.tracker
Terminal=false
Type=Application
Categories=Utility;Office;
DESKTOP

cat > "$flatpak_input/com.blevel.tracker.metainfo.xml" <<METAINFO
<?xml version="1.0" encoding="UTF-8"?>
<component type="desktop-application">
  <id>com.blevel.tracker</id>
  <name>bloody-level</name>
  <summary>Local pathology-report tracker</summary>
  <metadata_license>CC0-1.0</metadata_license>
  <project_license>MIT</project_license>
  <description>
    <p>Import, review, and compare blood-work reports locally with the original PDFs kept beside structured results.</p>
  </description>
  <launchable type="desktop-id">com.blevel.tracker.desktop</launchable>
  <releases>
    <release version="$version" />
  </releases>
</component>
METAINFO

cat > "$flatpak_input/com.blevel.tracker.json" <<'MANIFEST'
{
  "app-id": "com.blevel.tracker",
  "runtime": "org.gnome.Platform",
  "runtime-version": "48",
  "sdk": "org.gnome.Sdk",
  "command": "blevel-tracker",
  "finish-args": [
    "--share=ipc",
    "--socket=wayland",
    "--socket=fallback-x11",
    "--device=dri",
    "--filesystem=home"
  ],
  "modules": [
    {
      "name": "blevel-tracker",
      "buildsystem": "simple",
      "build-commands": [
        "install -Dm755 blevel-tracker /app/bin/blevel-tracker",
        "install -Dm755 libpdfium.so /app/bin/libpdfium.so",
        "install -Dm644 analytes.seed.json /app/bin/analytes.seed.json",
        "install -Dm644 com.blevel.tracker.desktop /app/share/applications/com.blevel.tracker.desktop",
        "install -Dm644 com.blevel.tracker.png /app/share/icons/hicolor/128x128/apps/com.blevel.tracker.png",
        "install -Dm644 com.blevel.tracker.metainfo.xml /app/share/metainfo/com.blevel.tracker.metainfo.xml"
      ],
      "sources": [
        { "type": "file", "path": "blevel-tracker" },
        { "type": "file", "path": "libpdfium.so" },
        { "type": "file", "path": "analytes.seed.json" },
        { "type": "file", "path": "com.blevel.tracker.desktop" },
        { "type": "file", "path": "com.blevel.tracker.png" },
        { "type": "file", "path": "com.blevel.tracker.metainfo.xml" }
      ]
    }
  ]
}
MANIFEST

flatpak-builder --force-clean --disable-rofiles-fuse --repo="$flatpak_repo" "$flatpak_build" "$flatpak_input/com.blevel.tracker.json"
flatpak build-bundle "$flatpak_repo" "$output_directory/$prefix.flatpak" com.blevel.tracker master \
  --runtime-repo=https://flathub.org/repo/flathub.flatpakrepo

echo "Packaged $prefix.AppImage, $prefix.deb, $prefix.rpm, $prefix.flatpak, and $prefix.zip"
