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

ANALYZER="$ROOT/resources/akron-analyzer"
if [[ ! -x "$ANALYZER" ]]; then
  echo "Missing executable Analyzer: $ANALYZER" >&2
  exit 1
fi
cp "$ANALYZER" "$RESOURCES/akron-runtime/akron-analyzer"
chmod 755 "$RESOURCES/akron-runtime/akron-analyzer"

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
EOF

/usr/bin/plutil -lint "$CONTENTS/Info.plist"

# Ensure the launch binaries remain executable immediately before signing.
chmod 755 "$MACOS/Akron" "$RESOURCES/akron-runtime/akron-analyzer"

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

# Re-assert permissions after signing and verify the final executable type.
chmod 755 "$MACOS/Akron" "$RESOURCES/akron-runtime/akron-analyzer"
/usr/bin/file "$MACOS/Akron"
/usr/bin/file "$RESOURCES/akron-runtime/akron-analyzer"

ZIP="$OUT/Akron-Native-macos-arm64.zip"
rm -f "$ZIP"
/usr/bin/ditto -c -k --sequesterRsrc --keepParent "$APP" "$ZIP"

# Produce a checksum alongside the exact ZIP distributed by CI.
/usr/bin/shasum -a 256 "$ZIP" | /usr/bin/tee "$OUT/Akron-Native-macos-arm64.zip.sha256"

printf 'Native macOS artifact: %s\n' "$ZIP"
printf 'Native macOS artifact checksum: %s\n' "$OUT/Akron-Native-macos-arm64.zip.sha256"
