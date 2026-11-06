#!/usr/bin/env bash
# Package a portable Linux build as a .zip. The portable app keeps its data in a
# `data/` folder next to the executable (via the `.portable` marker), leaving
# nothing in the home directory.
#
# Needs the release binary (cargo build --release -p dt-g1-studio-app) and the
# bundled resources (scripts/vendor.sh). Usage: scripts/portable.sh
set -euo pipefail

CONFIG="${1:-release}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VER="$(grep -oP '"version"\s*:\s*"\K[^"]+' "$ROOT/src-tauri/tauri.conf.json" | head -1)"
EXE="$ROOT/target/$CONFIG/dt-g1-studio-app"
RES="$ROOT/src-tauri/resources"
[ -f "$EXE" ] || { echo "missing $EXE - run: cargo build --$CONFIG -p dt-g1-studio-app" >&2; exit 1; }
[ -d "$RES" ] || { echo "missing resources - run: scripts/vendor.sh" >&2; exit 1; }

STAGE="$(mktemp -d)"
APP="$STAGE/DT-G1-Studio"
trap 'rm -rf "$STAGE"' EXIT
mkdir -p "$APP"

cp "$EXE" "$APP/dt-g1-studio"
chmod +x "$APP/dt-g1-studio"
cp -r "$RES" "$APP/resources"
printf 'Portable mode: data is kept in the data/ folder next to this file.\n' > "$APP/.portable"
printf 'DT G1 Studio (portable)\nRun ./dt-g1-studio. Data is kept in the data/ folder beside it.\n' > "$APP/README.txt"

OUT="$ROOT/DT-G1-Studio-$VER-portable-linux-x64.zip"
rm -f "$OUT"
( cd "$STAGE" && zip -qr "$OUT" "DT-G1-Studio" )
echo "portable -> $OUT"
