#!/usr/bin/env bash
# Assemble the bundled tool resources into src-tauri/resources (gitignored),
# Linux variant. Mirrors scripts/vendor.ps1. The map compiler has no upstream
# Linux build, so it is skipped (compiling maps is Windows-only for now).
#
# Usage:  scripts/vendor.sh [platform]   (default linux-x64)
set -euo pipefail

PLATFORM="${1:-linux-x64}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
RES="$ROOT/src-tauri/resources"
TMP="$(mktemp -d)"
GX_VERSION="0.8.0-rc"
trap 'rm -rf "$TMP"' EXIT
mkdir -p "$RES"

say() { echo "[vendor] $*"; }

# --- gx toolchain -----------------------------------------------------------
say "fetching @glyphix/cli-$PLATFORM@$GX_VERSION via npm pack"
GXDST="$RES/gx"; rm -rf "$GXDST"; mkdir -p "$GXDST"
( cd "$TMP" && npm pack "@glyphix/cli-$PLATFORM@$GX_VERSION" >/dev/null )
tgz="$(ls "$TMP"/*.tgz | head -1)"
tar -xzf "$tgz" -C "$TMP"
cp -r "$TMP/package/bin" "$GXDST/bin"
cp -r "$TMP/package/share" "$GXDST/share"
# Drop the emulator binary (never used for a build); keep shared libs for safety.
rm -f "$GXDST/bin/glyphix-emu" "$GXDST/bin/glyphix-emu.exe" 2>/dev/null || true
chmod +x "$GXDST"/bin/* 2>/dev/null || true
# Embed the device profile (gx searches share/glyphix/devices next to the binary).
DEVDST="$GXDST/share/glyphix/devices"; mkdir -p "$DEVDST"
cp "$ROOT/src-tauri/device/dtg1-c175r.yml" "$DEVDST/"
cp "$ROOT/src-tauri/device/image-convert-pal.scm" "$DEVDST/"

# --- map compiler (no Linux build upstream) ---------------------------------
say "skipping map compiler (no $PLATFORM build upstream)"
MAPVER=""

# --- launcher (platform-independent .pkg) -----------------------------------
say "downloading launcher (PurrLaunch)"
LAUNDST="$RES/launcher"; rm -rf "$LAUNDST"; mkdir -p "$LAUNDST"
LREL="$(curl -fsSL -H 'User-Agent: dt-g1-studio' https://api.github.com/repos/purrrock/PurrLaunch/releases/latest)"
LAUNVER="$(printf '%s' "$LREL" | grep -oP '"tag_name":\s*"\K[^"]+' | head -1)"
LURL="$(printf '%s' "$LREL" | grep -oP '"browser_download_url":\s*"\K[^"]+\.pkg' | head -1)"
curl -fsSL -H 'User-Agent: dt-g1-studio' "$LURL" -o "$LAUNDST/com.xfaith.woodenfish.pkg"

# --- manifest ---------------------------------------------------------------
cat > "$RES/manifest.json" <<EOF
{ "gx": "$GX_VERSION", "map-compiler": "$MAPVER", "launcher": "$LAUNVER", "platform": "$PLATFORM" }
EOF

say "done -> $RES"
