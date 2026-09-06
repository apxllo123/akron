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

cp "$ROOT/macos/Info.plist" "$CONTENTS/Info.plist"
cp -R "$ROOT/dist-renderer/." "$RESOURCES/ui/"

ANALYZER="$ROOT/resources/akron-analyzer"
if [[ ! -x "$ANALYZER" ]]; then
  echo "Missing executable Analyzer: $ANALYZER" >&2
  exit 1
fi
cp "$ANALYZER" "$RESOURCES/akron-runtime/akron-analyzer"
chmod 755 "$MACOS/Akron" "$RESOURCES/akron-runtime/akron-analyzer"

/usr/bin/plutil -lint "$CONTENTS/Info.plist"
/usr/bin/codesign --force --deep --sign - "$APP"
/usr/bin/codesign --verify --verbose=2 "$APP"

rm -f "$OUT/Akron-macos-arm64.zip"
/usr/bin/ditto -c -k --sequesterRsrc --keepParent "$APP" "$OUT/Akron-macos-arm64.zip"

echo "Native macOS artifact: $OUT/Akron-macos-arm64.zip"
