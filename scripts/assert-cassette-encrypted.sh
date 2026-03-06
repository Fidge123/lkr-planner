#!/usr/bin/env bash

set -euo pipefail

mode="encrypted"
if [ "${1:-}" = "--encrypted" ]; then
  shift
elif [ "${1:-}" = "--decrypted" ]; then
  mode="decrypted"
  shift
fi

if [ "$#" -lt 1 ]; then
  echo "Usage: assert-cassette-encrypted.sh [--encrypted|--decrypted] <cassette-path-or-dir> [...]" >&2
  exit 1
fi

declare -a cassette_paths=()

for input_path in "$@"; do
  if [ -d "$input_path" ]; then
    while IFS= read -r -d '' cassette_path; do
      cassette_paths+=("$cassette_path")
    done < <(find "$input_path" -type f -name '*.json' -print0 | sort -z)
    continue
  fi

  if [ -f "$input_path" ]; then
    cassette_paths+=("$input_path")
    continue
  fi

  echo "Cassette path not found: $input_path" >&2
  exit 1
done

if [ "${#cassette_paths[@]}" -eq 0 ]; then
  echo "No cassette files found in the provided paths." >&2
  exit 1
fi

for cassette_path in "${cassette_paths[@]}"; do
  if [ "$mode" = "encrypted" ]; then
    if grep -Iq . "$cassette_path"; then
      echo "Cassette appears to be plaintext before git-crypt unlock: $cassette_path" >&2
      exit 1
    fi

    if LC_ALL=C grep -qa '"interactions"' "$cassette_path"; then
      echo "Cassette still exposes JSON markers before git-crypt unlock: $cassette_path" >&2
      exit 1
    fi

    continue
  fi

  if ! grep -Iq . "$cassette_path"; then
    echo "Cassette is still unreadable after git-crypt unlock: $cassette_path" >&2
    exit 1
  fi

  if ! LC_ALL=C grep -qa '"interactions"' "$cassette_path"; then
    echo "Cassette does not expose the expected JSON markers after unlock: $cassette_path" >&2
    exit 1
  fi
done

echo "Verified ${#cassette_paths[@]} cassette file(s) in ${mode} mode."
