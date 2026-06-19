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

# A browser is installed when Playwright has written its INSTALLATION_COMPLETE
# marker into a "<engine>-<revision>" directory. This is engine- and
# platform-agnostic, so the same check works for chromium and webkit.
browser_installed() {
  for marker in "$browsers_dir"/"$1"-*/INSTALLATION_COMPLETE; do
    if [ -f "$marker" ]; then
      return 0
    fi
  done
  return 1
}

if ! browser_installed chromium; then
  warn "Playwright chromium not installed. Run: bunx playwright install chromium"
fi

if ! browser_installed webkit; then
  warn "Playwright webkit not installed. Run: bunx playwright install webkit"
fi

exit 0
