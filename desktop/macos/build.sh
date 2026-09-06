#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="$ROOT/dist-native"
APP="$OUT/Akron.app"
CONTENTS="$APP/Contents"
RESOURCES="$CONTENTS/Resources"
MACOS="$CONTENTS/MacOS"

rm -rf "$OUT"
mkdir -p "$MACOS" "$RESOURCES/ui" "$RESOURCES/akron-runtime"

swiftc \
  -O \
  -whole-module-optimization \
  -target arm64-apple-macosx15.0 \
  -framework AppKit \
  -framework WebKit \
  "$ROOT/macos/AkronNative.swift" \
  -o "$MACOS/Akron"

chmod 755 "$MACOS/Akron"

cp "$ROOT/macos/Info.plist" "$CONTENTS/Info.plist"
cp -R "$ROOT/dist-renderer/." "$RESOURCES/ui/"

# Native builds use the same runtime staging path as Electron builds so the
# packaged app always contains both Rust executors required by the UI bridge.
node "$ROOT/scripts/stage-runtime.mjs"

ANALYZER="$ROOT/resources/akron-analyzer"
ADAPTER="$ROOT/resources/akron-adapter"
if [[ ! -x "$ANALYZER" ]]; then
  echo "Missing executable Analyzer: $ANALYZER" >&2
  exit 1
fi
if [[ ! -x "$ADAPTER" ]]; then
  echo "Missing executable Adapter: $ADAPTER" >&2
  exit 1
fi
cp "$ANALYZER" "$RESOURCES/akron-runtime/akron-analyzer"
cp "$ADAPTER" "$RESOURCES/akron-runtime/akron-adapter"
chmod 755 "$RESOURCES/akron-runtime/akron-analyzer" "$RESOURCES/akron-runtime/akron-adapter"

# The native macOS build intentionally does not depend on Electron.
rm -f "$RESOURCES/Akron.icns"
/usr/bin/plutil -delete CFBundleIconFile "$CONTENTS/Info.plist" 2>/dev/null || true

BUILD_SHA="${GITHUB_SHA:-local}"
BUILD_VERSION="$(/usr/bin/plutil -extract CFBundleShortVersionString raw "$CONTENTS/Info.plist")"
cat > "$RESOURCES/Akron-Build.txt" <<EOF
Akron native macOS build
Host: AppKit + WKWebView
Architecture: arm64
Electron runtime: not used
Icon: pending dedicated Akron asset
Validation profile: native-macos-v3
Build version: $BUILD_VERSION
Git SHA: $BUILD_SHA
Distribution: ZIP + DMG
Runtime: Analyzer + Adapter
EOF

/usr/bin/plutil -lint "$CONTENTS/Info.plist"

chmod 755 "$MACOS/Akron" "$RESOURCES/akron-runtime/akron-analyzer" "$RESOURCES/akron-runtime/akron-adapter"

SIGNING_IDENTITY="${APPLE_SIGNING_IDENTITY:--}"
if [[ "$SIGNING_IDENTITY" == "-" ]]; then
  echo "Code signing mode: ad-hoc"
  /usr/bin/codesign --force --sign - "$APP"
else
  echo "Code signing mode: identity $SIGNING_IDENTITY"
  /usr/bin/codesign --force --options runtime --sign "$SIGNING_IDENTITY" "$APP"
fi

/usr/bin/codesign --verify --deep --strict --verbose=2 "$APP"
/usr/bin/codesign --display --verbose=4 "$APP" || true

chmod 755 "$MACOS/Akron" "$RESOURCES/akron-runtime/akron-analyzer" "$RESOURCES/akron-runtime/akron-adapter"
/usr/bin/file "$MACOS/Akron"
/usr/bin/file "$RESOURCES/akron-runtime/akron-analyzer"
/usr/bin/file "$RESOURCES/akron-runtime/akron-adapter"

# Keep the ZIP for scripting/debugging and add a native Finder-friendly DMG.
ZIP="$OUT/Akron-Native-macos-arm64.zip"
rm -f "$ZIP"
/usr/bin/ditto -c -k --sequesterRsrc --keepParent "$APP" "$ZIP"
/usr/bin/shasum -a 256 "$ZIP" | /usr/bin/tee "$OUT/Akron-Native-macos-arm64.zip.sha256"

DMG="$OUT/Akron-Native-macos-arm64.dmg"
DMG_STAGE="$OUT/.dmg-stage"
rm -rf "$DMG_STAGE" "$DMG"
mkdir -p "$DMG_STAGE"
/usr/bin/ditto "$APP" "$DMG_STAGE/Akron.app"
ln -s /Applications "$DMG_STAGE/Applications"

# UDZO is a standard compressed read-only HFS+/APFS-compatible disk image format.
/usr/bin/hdiutil create \
  -volname "Akron" \
  -srcfolder "$DMG_STAGE" \
  -ov \
  -format UDZO \
  "$DMG"

rm -rf "$DMG_STAGE"

# Validate that the image can be mounted and contains exactly the expected app.
MOUNT_INFO="$(/usr/bin/hdiutil attach "$DMG" -nobrowse -noautoopen)"
MOUNT_PATH="$(printf '%s\n' "$MOUNT_INFO" | awk -F'\t' '/\/Volumes\// {print $NF; exit}')"
if [[ -z "$MOUNT_PATH" ]]; then
  echo "DMG validation failed: could not mount $DMG" >&2
  exit 1
fi
trap '/usr/bin/hdiutil detach "$MOUNT_PATH" -quiet >/dev/null 2>&1 || true' EXIT

test -d "$MOUNT_PATH/Akron.app"
test -d "$MOUNT_PATH/Applications"
test -x "$MOUNT_PATH/Akron.app/Contents/MacOS/Akron"
test -x "$MOUNT_PATH/Akron.app/Contents/Resources/akron-runtime/akron-analyzer"
test -x "$MOUNT_PATH/Akron.app/Contents/Resources/akron-runtime/akron-adapter"
/usr/bin/codesign --verify --deep --strict --verbose=2 "$MOUNT_PATH/Akron.app"
/usr/bin/ditto --rsrc "$MOUNT_PATH/Akron.app/Contents/Resources/Akron-Build.txt" /tmp/akron-dmg-marker.txt
/usr/bin/hdiutil detach "$MOUNT_PATH" -quiet
trap - EXIT
rm -f /tmp/akron-dmg-marker.txt

/usr/bin/shasum -a 256 "$DMG" | /usr/bin/tee "$OUT/Akron-Native-macos-arm64.dmg.sha256"

printf 'Native macOS ZIP: %s\n' "$ZIP"
printf 'Native macOS DMG: %s\n' "$DMG"
printf 'ZIP checksum: %s\n' "$OUT/Akron-Native-macos-arm64.zip.sha256"
printf 'DMG checksum: %s\n' "$OUT/Akron-Native-macos-arm64.dmg.sha256"
