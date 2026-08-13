#!/usr/bin/env bash
set -euo pipefail

NAME="rmon"
PREFIX="${PREFIX:-$HOME/.local/bin}"
EXT=""
case "$(uname -s)" in
  MINGW*|MSYS*|CYGWIN*) EXT=".exe" ;;
esac
BIN="$PREFIX/$NAME$EXT"

GREEN=$'\033[32m'
CYAN=$'\033[36m'
YELLOW=$'\033[33m'
RESET=$'\033[0m'

say() { printf '%s\n' "${CYAN}==>${RESET} $*"; }
ok()  { printf '%s\n' "${GREEN}==>${RESET} $*"; }

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

say "RMon uninstaller"

if [ -f "$BIN" ]; then
  rm -f "$BIN"
  ok "Removed ${BIN}"
else
  printf '%s\n' "${YELLOW}==>${RESET} $NAME not found at $BIN (nothing to do)."
fi

if [ -d "$SCRIPT_DIR/target" ]; then
  REPLY="n"
  read -r -p "Remove build artifacts in target/? [y/N] " -n 1 REPLY 2>/dev/null || true
  echo
  case "$REPLY" in
    y|Y)
      rm -rf "$SCRIPT_DIR/target"
      ok "Removed $SCRIPT_DIR/target"
      ;;
    *)
      say "Skipped build artifacts."
      ;;
  esac
fi

ok "Uninstall complete."
