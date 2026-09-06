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

ICON_SOURCE="$ROOT/../resources/icon.png"
if [[ ! -f "$ICON_SOURCE" ]]; then
  ICON_SOURCE="$ROOT/../resources/icon.jpeg"
fi
ICONSET="$OUT/Akron.iconset"
ICON_PNG="$OUT/Akron-source.png"
ICON_ICNS="$RESOURCES/Akron.icns"
if [[ ! -f "$ICON_SOURCE" ]]; then
  echo "Missing Akron icon source: expected resources/icon.png or resources/icon.jpeg" >&2
  exit 1
fi
rm -rf "$ICONSET" "$ICON_PNG" "$ICON_ICNS"
mkdir -p "$ICONSET"
/usr/bin/sips -s format png "$ICON_SOURCE" --out "$ICON_PNG" >/dev/null

for size in 16 32 128 256 512; do
  /usr/bin/sips -z "$size" "$size" "$ICON_PNG" --out "$ICONSET/icon_${size}x${size}.png" >/dev/null
  doubled=$((size * 2))
  /usr/bin/sips -z "$doubled" "$doubled" "$ICON_PNG" --out "$ICONSET/icon_${size}x${size}@2x.png" >/dev/null
done

/usr/bin/iconutil -c icns "$ICONSET" -o "$ICON_ICNS"
test -s "$ICON_ICNS"
/usr/bin/plutil -replace CFBundleIconFile -string Akron "$CONTENTS/Info.plist"

BUILD_SHA="${GITHUB_SHA:-local}"
BUILD_VERSION="$(/usr/bin/plutil -extract CFBundleShortVersionString raw "$CONTENTS/Info.plist")"
cat > "$RESOURCES/Akron-Build.txt" <<EOF
Akron native macOS build
Host: AppKit + WKWebView
Architecture: arm64
Electron runtime: not used
Icon: Akron.icns generated from $ICON_SOURCE
Validation profile: native-macos-v4
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
/usr/bin/file "$ICON_ICNS"
/usr/bin/file "$RESOURCES/akron-runtime/akron-analyzer"
/usr/bin/file "$RESOURCES/akron-runtime/akron-adapter"

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
/usr/bin/hdiutil create \
  -volname "Akron" \
  -srcfolder "$DMG_STAGE" \
  -ov \
  -format UDZO \
  "$DMG"
rm -rf "$DMG_STAGE"

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
test -f "$MOUNT_PATH/Akron.app/Contents/Resources/Akron.icns"
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
