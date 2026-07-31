#!/bin/bash

set -euo pipefail

if [ "${CLAUDE_CODE_REMOTE:-}" != "true" ]; then
  exit 0
fi

cd "${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel)}"

bun install

if ! command -v git-crypt >/dev/null 2>&1; then
  sudo apt-get install -y git-crypt || {
    sudo apt-get update
    sudo apt-get install -y git-crypt
  }
fi

bash scripts/unlock-cassettes.sh || echo "Cassettes stay locked; the Daylite VCR tests will fail in this session."

cargo fetch --manifest-path src-tauri/Cargo.toml

uvx radicale --version >/dev/null 2>&1 || true
