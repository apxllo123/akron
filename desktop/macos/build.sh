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

# Swift normally creates an executable here, but enforce the mode explicitly so
# the packaged app cannot lose execute permission during later file operations.
chmod 755 "$MACOS/Akron"

cp "$ROOT/macos/Info.plist" "$CONTENTS/Info.plist"
cp -R "$ROOT/dist-renderer/." "$RESOURCES/ui/"

ANALYZER="$ROOT/resources/akron-analyzer"
if [[ ! -x "$ANALYZER" ]]; then
  echo "Missing executable Analyzer: $ANALYZER" >&2
  exit 1
fi
cp "$ANALYZER" "$RESOURCES/akron-runtime/akron-analyzer"
chmod 755 "$RESOURCES/akron-runtime/akron-analyzer"

# Preserve the icon used by the previous Electron distribution until a
# dedicated Akron .icns asset is added.
ELECTRON_ICON="$ROOT/node_modules/electron/dist/Electron.app/Contents/Resources/electron.icns"
if [[ -f "$ELECTRON_ICON" ]]; then
  cp "$ELECTRON_ICON" "$RESOURCES/Akron.icns"
  /usr/bin/plutil -replace CFBundleIconFile -string "Akron.icns" "$CONTENTS/Info.plist"
fi

# Make the native build unmistakable when extracted from GitHub Actions.
cat > "$RESOURCES/Akron-Build.txt" <<'EOF'
Akron native macOS build
Host: AppKit + WKWebView
Architecture: arm64
Electron runtime: not used
EOF

/usr/bin/plutil -lint "$CONTENTS/Info.plist"

# Re-assert executable modes immediately before signing and archive creation.
chmod 755 "$MACOS/Akron" "$RESOURCES/akron-runtime/akron-analyzer"

/usr/bin/codesign --force --sign - "$APP"
/usr/bin/codesign --verify --verbose=2 "$APP"

# Code signing must not leave the launch binaries non-executable.
chmod 755 "$MACOS/Akron" "$RESOURCES/akron-runtime/akron-analyzer"
/usr/bin/codesign --verify --verbose=2 "$APP"

ZIP="$OUT/Akron-Native-macos-arm64.zip"
rm -f "$ZIP"
/usr/bin/ditto -c -k --sequesterRsrc --keepParent "$APP" "$ZIP"

echo "Native macOS artifact: $ZIP"
