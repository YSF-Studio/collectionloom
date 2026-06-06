#!/usr/bin/env bash
# Build, sign, notarize, and staple CollectionLoom for macOS distribution.
# Requires: Apple Developer ID cert, notarytool credentials in keychain.
#
# Usage:
#   export APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAMID)"
#   export APPLE_ID="you@example.com"
#   export APPLE_TEAM_ID="TEAMID"
#   export APPLE_APP_PASSWORD="@keychain:AC_PASSWORD"
#   ./scripts/notarize-macos.sh

set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

: "${APPLE_SIGNING_IDENTITY:?Set APPLE_SIGNING_IDENTITY}"
: "${APPLE_ID:?Set APPLE_ID}"
: "${APPLE_TEAM_ID:?Set APPLE_TEAM_ID}"

echo "Building CollectionLoom release…"
npm run build
(
  cd src-tauri
  export MACOSX_DEPLOYMENT_TARGET=11.0
  cargo tauri build --bundles dmg
)

APP="src-tauri/target/release/bundle/macos/CollectionLoom.app"
DMG="src-tauri/target/release/bundle/dmg/CollectionLoom_0.1.0_aarch64.dmg"

if [[ ! -d "$APP" ]]; then
  APP=$(find src-tauri/target -name 'CollectionLoom.app' -type d | head -1)
fi

echo "Signing app: $APP"
codesign --force --deep --options runtime \
  --entitlements src-tauri/entitlements.plist \
  --sign "$APPLE_SIGNING_IDENTITY" \
  "$APP"

echo "Creating notarization zip…"
ZIP="/tmp/collectionloom-notarize.zip"
ditto -c -k --keepParent "$APP" "$ZIP"

echo "Submitting to Apple notary service…"
xcrun notarytool submit "$ZIP" \
  --apple-id "$APPLE_ID" \
  --team-id "$APPLE_TEAM_ID" \
  --password "${APPLE_APP_PASSWORD:-@keychain:AC_PASSWORD}" \
  --wait

echo "Stapling ticket…"
xcrun stapler staple "$APP"
if [[ -f "$DMG" ]]; then
  xcrun stapler staple "$DMG" || true
fi

echo "Done. Signed and notarized: $APP"
spctl -a -vv "$APP" || true
