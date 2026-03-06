#!/usr/bin/env bash

set -euo pipefail

cassette_path="${1:?Usage: assert-cassette-encrypted.sh <cassette-path>}"

if [ ! -f "$cassette_path" ]; then
  echo "Cassette file not found: $cassette_path" >&2
  exit 1
fi

if grep -Iq . "$cassette_path"; then
  echo "Cassette appears to be plaintext before git-crypt unlock: $cassette_path" >&2
  exit 1
fi

if LC_ALL=C grep -qa '"interactions"' "$cassette_path"; then
  echo "Cassette still exposes JSON markers before git-crypt unlock: $cassette_path" >&2
  exit 1
fi

echo "Cassette is not readable before git-crypt unlock: $cassette_path"
