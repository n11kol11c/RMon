#!/usr/bin/env bash
set -euo pipefail

NAME="rmon"
PREFIX="${PREFIX:-$HOME/.local/bin}"

GREEN=$'\033[32m'
CYAN=$'\033[36m'
YELLOW=$'\033[33m'
RED=$'\033[31m'
BOLD=$'\033[1m'
RESET=$'\033[0m'

say()  { printf '%s\n' "${CYAN}==>${RESET} $*"; }
ok()   { printf '%s\n' "${GREEN}==>${RESET} $*"; }
warn() { printf '%s\n' "${YELLOW}==>${RESET} $*"; }
die()  { printf '%s\n' "${RED}==>${RESET} $*" >&2; exit 1; }

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

say "RMon installer"

OS="$(uname -s)"
ARCH="$(uname -m)"
case "$OS" in
  Linux)   OS_NAME="Linux" ;;
  Darwin)  OS_NAME="macOS" ;;
  MINGW*|MSYS*|CYGWIN*) OS_NAME="Windows (Git Bash/MSYS)" ;;
  *)       OS_NAME="$OS" ;;
esac
say "Platform: ${BOLD}${OS_NAME}${RESET} / ${BOLD}${ARCH}${RESET}"

if ! command -v cargo >/dev/null 2>&1; then
  die "cargo not found. Install Rust first: https://rustup.rs"
fi

say "Building release binary..."
(
  cd "$SCRIPT_DIR"
  cargo build --release
)

mkdir -p "$PREFIX"
cp "$SCRIPT_DIR/target/release/$NAME" "$PREFIX/$NAME"
chmod +x "$PREFIX/$NAME"

ok "Installed $NAME to ${BOLD}$PREFIX/$NAME${RESET}"

case ":$PATH:" in
  *":$PREFIX:"*) ok "$PREFIX is already on your PATH." ;;
  *)
    warn "$PREFIX is not on your PATH."
    printf '    Add it with:  %s\n' \
      "echo 'export PATH=\"\$PATH:$PREFIX\"' >> ~/.zshrc" \
      "echo 'export PATH=\"\$PATH:$PREFIX\"' >> ~/.bashrc"
    ;;
esac

ok "Done. Run '${BOLD}$NAME${RESET}' to start."
