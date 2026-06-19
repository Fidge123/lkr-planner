#!/bin/sh
# Fast, non-blocking development environment check run at Claude Code session
# start. It warns about missing tools but always exits 0 so it never blocks or
# slows session startup. It is run directly by the shell (not via `bun`) so a
# missing `bun` can still be reported, and it uses only POSIX shell builtins so
# it works even with an empty PATH.

missing=0

warn() {
  echo "WARN: $1" >&2
  missing=1
}

if ! command -v bun >/dev/null 2>&1; then
  warn "bun not found on PATH. Install Bun: https://bun.sh"
fi

if ! command -v cargo >/dev/null 2>&1; then
  warn "cargo not found on PATH. Install the Rust toolchain: https://rustup.rs"
fi

browsers_dir="${PLAYWRIGHT_BROWSERS_PATH:-$HOME/.cache/ms-playwright}"
chromium_found=0
for chrome in \
  "$browsers_dir"/chromium-*/chrome-linux*/chrome \
  "$browsers_dir"/chromium-*/chrome-mac*/Chromium.app/Contents/MacOS/Chromium; do
  if [ -x "$chrome" ]; then
    chromium_found=1
    break
  fi
done
if [ "$chromium_found" -eq 0 ]; then
  warn "Playwright chromium not installed. Run: bunx playwright install chromium"
fi

exit 0
