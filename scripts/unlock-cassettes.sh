#!/usr/bin/env bash

set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cassette_dir="$repo_root/tests/cassettes"

if [ ! -d "$cassette_dir" ]; then
  exit 0
fi

if bash "$repo_root/scripts/assert-cassette-encrypted.sh" --decrypted "$cassette_dir" >/dev/null 2>&1; then
  exit 0
fi

if ! command -v git-crypt >/dev/null 2>&1; then
  echo "Cassettes are locked and git-crypt is not installed. Install git-crypt to run the VCR tests." >&2
  exit 1
fi

if [ -z "${GIT_CRYPT_KEY_B64:-}" ]; then
  echo "Cassettes are locked and GIT_CRYPT_KEY_B64 is not set. Run 'git-crypt unlock <key>' to run the VCR tests." >&2
  exit 1
fi

key_file="$(mktemp)"
trap 'rm -f "$key_file"' EXIT

if base64 --help 2>&1 | grep -q -- '--decode'; then
  printf '%s' "$GIT_CRYPT_KEY_B64" | base64 --decode > "$key_file"
else
  printf '%s' "$GIT_CRYPT_KEY_B64" | base64 -D > "$key_file"
fi

git-crypt unlock "$key_file"

bash "$repo_root/scripts/assert-cassette-encrypted.sh" --decrypted "$cassette_dir" >/dev/null
echo "Unlocked git-crypt cassettes."
